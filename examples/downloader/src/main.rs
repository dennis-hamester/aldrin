use aldrin::core::tokio::TokioTransport;
use aldrin::core::ObjectUuid;
use aldrin::{Client, Handle, Promise, UnboundSender};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use downloader::{Chunk, Downloader, DownloaderDownloadArgs, DownloaderFunction, DownloaderProxy};
use sha2::{Digest, Sha256};
use std::convert::Infallible;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::signal;

const BUS_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 24940);
const CHUNK_SIZE: usize = 64 * 1024;
const CAPACITY: u32 = 32;

// The generated code can be inspected using rustdoc. Just run
// `cargo doc --document-private-items --open` and look at the `downloader` module.
aldrin::generate!("src/downloader.aldrin", introspection_if = "introspection");

/// Downloader example.
#[derive(Parser)]
struct Args {
    /// Address of the broker to connect to.
    #[clap(short, long, default_value_t = BUS_DEFAULT)]
    bus: SocketAddr,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
enum Command {
    /// Run a downloader server to receive files.
    Server,

    /// List available downloader servers.
    List,

    /// Upload a file to a server.
    Upload(UploadArgs),
}

#[derive(Parser)]
struct UploadArgs {
    /// UUID of the server to use.
    ///
    /// If this is not specified, then the first server is used that is found.
    #[clap(short, long)]
    server: Option<ObjectUuid>,

    /// Path of the file to upload.
    path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Connecting to broker at {}.", args.bus);

    let stream = TcpStream::connect(&args.bus)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;

    // Setting nodelay on the TCP socket can vastly improve latencies as Aldrin messages are
    // typically small.
    stream.set_nodelay(true)?;

    let transport = TokioTransport::new(stream);

    let client = Client::connect(transport)
        .await
        .with_context(|| anyhow!("failed to connect to broker at {}", args.bus))?;
    let handle = client.handle().clone();
    let join = tokio::spawn(client.run());

    #[cfg(feature = "introspection")]
    {
        downloader::register_introspection(&handle)?;
        handle.submit_introspection()?;
    }

    match args.cmd {
        Command::Server => server(&handle).await?,
        Command::List => list(&handle).await?,
        Command::Upload(args) => upload(&handle, args).await?,
    }

    println!("Shutting down connection to the broker.");

    handle.shutdown();
    join.await
        .with_context(|| anyhow!("failed to shut down client"))?
        .with_context(|| anyhow!("failed to shut down client"))?;

    Ok(())
}

async fn server(bus: &Handle) -> Result<()> {
    let object = bus.create_object(ObjectUuid::new_v4()).await?;
    let mut downloader = Downloader::new(&object).await?;

    println!("Starting downloader server {}.", object.id().uuid);

    loop {
        tokio::select! {
            function = downloader.next_call() => {
                match function {
                    Some(Ok(DownloaderFunction::Download(args, promise))) => {
                        tokio::spawn(download(args, promise));
                    }

                    // Function calls can be invalid either because their id is unknown or because
                    // the arguments to the call failed to deserialize.
                    Some(Err(e)) => {
                        println!("Received an invalid function call: {e}.");
                        continue;
                    }

                    // `None` is returned when the service is destroyed or when the connection to
                    // the broker shuts down.
                    None => break Err(anyhow!("broker shut down")),
                }
            }

            signal = signal::ctrl_c() => {
                signal.with_context(|| anyhow!("failed to listen for CTRL-C"))?;
                break Ok(());
            }
        }
    }
}

async fn download(
    args: DownloaderDownloadArgs,
    promise: Promise<UnboundSender<Chunk>, Infallible>,
) {
    println!("Downloading `{}` ({} bytes).", args.name, args.size);

    match download_impl(args.size, promise).await {
        Ok(()) => println!("Download of `{}` finished successfully.", args.name),
        Err(e) => println!("Download of `{}` failed: {}.", args.name, e),
    }
}

async fn download_impl(
    expected_size: u64,
    promise: Promise<UnboundSender<Chunk>, Infallible>,
) -> Result<()> {
    // When creating a channel, one of the channel ends must be immediately claimed by the local
    // client and only the other end can be sent to another client. Here, we claim the receiver,
    // which also requires specifying a capacity.
    let (sender, receiver) = promise
        .client()
        .create_channel()
        .claim_receiver(CAPACITY)
        .await?;

    // Fulfill the function call and send back the sender. Channel ends must be unbound from the
    // client they were created from first.
    promise.ok(sender.unbind())?;

    // Wait for the channel to be established. This will block until the sender is claimed by the
    // other client.
    let mut receiver = receiver.establish().await?;

    let mut sha256 = Sha256::new();
    let mut size = 0u64;

    loop {
        // Receivers are asynchronous streams of the item type. Here, `next_item()` returns
        // `Result<Option<Chunk>>`. An implementation of the `Stream` trait from the futures crate
        // is also provided.
        match receiver.next_item().await? {
            Some(Chunk::Data(data)) => {
                sha256.update(&data);
                size += data.len() as u64;
            }

            Some(Chunk::Done(expected_sha256)) => {
                if size != expected_size {
                    break Err(anyhow!(
                        "size mismatch; expected {expected_size}, got {size}"
                    ));
                }

                if sha256.finalize()[..] != expected_sha256 {
                    break Err(anyhow!("SHA-256 mismatch"));
                }

                break Ok(());
            }

            None => break Err(anyhow!("channel closed unexpectedly")),
        }
    }
}

async fn list(bus: &Handle) -> Result<()> {
    // `Discoverer`s are used to find objects on the bus that implement some specific set of
    // services. Every call to `add_object` registers interest in one type of object.
    let mut discoverer = bus
        .create_discoverer()
        .any((), [DownloaderProxy::UUID])
        .build_current_only()
        .await?;

    let mut found = false;

    // The discoverer will emit events for every object it has found on the bus that matches the
    // criteria. Through the event, we can get the object's id and the relevant service ids.
    while let Some(event) = discoverer.next_event().await {
        println!("Found downloader server {}.", event.object_id().uuid);
        found = true;
    }

    if !found {
        println!("No downloader servers found.");
    }

    Ok(())
}

async fn upload(bus: &Handle, args: UploadArgs) -> Result<()> {
    let downloader = get_downloader(bus, args.server).await?;

    let name = args
        .path
        .file_name()
        .ok_or_else(|| anyhow!("failed to determine file name of `{}`", args.path.display()))?
        .to_string_lossy();

    let mut file = File::open(&args.path)
        .with_context(|| anyhow!("failed to open `{}`", args.path.display()))?;

    let size = file
        .metadata()
        .with_context(|| anyhow!("failed to determine size of `{}`", args.path.display()))?
        .len();

    // When channels ends (here the sender) are sent over an Aldrin bus, they are not bound to any
    // specific client. The type of `sender` here is `UnboundSender<Chunk>`.
    let sender = downloader
        .download(&DownloaderDownloadArgs {
            name: name.into_owned(),
            size,
        })
        .await??;

    // In order to use the sender, it must be bound to a client and claimed. The `claim()` function
    // on `UnboundSender` will perform both steps and return a `Sender<Chunk>`. This will also
    // unblock the receiver's call to `PendingReceiver::established()`.
    let mut sender = sender.claim(bus.clone()).await?;

    let mut buf = vec![0; CHUNK_SIZE];
    let mut sha256 = Sha256::new();
    let time = Instant::now();

    println!("Uploading file `{}` ({} bytes).", args.path.display(), size);

    loop {
        match file.read(&mut buf)? {
            0 => break,

            n => {
                sha256.update(&buf[..n]);

                // Sending items can block if the channel's capacity is exhausted. When that
                // happens, the receiver must first remove some items from its end. There is also an
                // optional `Sink` trait implementation from the futures crate.
                sender
                    .send_item(&Chunk::Data(buf[..n].to_vec().into()))
                    .await?;
            }
        }
    }

    let sha256 = sha256.finalize();
    sender
        .send_item(&Chunk::Done(sha256.to_vec().into()))
        .await?;

    let duration = time.elapsed();
    let mibps = size as f64 / duration.as_secs_f64() / 1024.0 / 1024.0;

    println!(
        "Upload finished in {} milliseconds ({:.2} MiB/s).",
        duration.as_millis(),
        mibps
    );

    Ok(())
}

async fn get_downloader(bus: &Handle, object_uuid: Option<ObjectUuid>) -> Result<DownloaderProxy> {
    // Find either a specific object (when `object_uuid` is `Some`) or any object implementing the
    // Downloader service. The `find_object` function is a convenience wrapper for `Discoverer`,
    // which is more concise when looking for a single object as a one-shot operation.
    let (_, [service_id]) = bus
        .find_object(object_uuid, &[DownloaderProxy::UUID])
        .await?
        .ok_or_else(|| anyhow!("downloader server not found"))?;

    let downloader = DownloaderProxy::new(bus, service_id).await?;
    Ok(downloader)
}
