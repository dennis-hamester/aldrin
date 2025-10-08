use aldrin::core::tokio::TokioTransport;
use aldrin::core::ObjectUuid;
use aldrin::{Client, Handle};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use echo::{Echo, EchoCall, EchoEchoAllError, EchoEchoError, EchoEvent, EchoProxy};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpStream;
use tokio::signal;

const BUS_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 24940);

// The generated code can be inspected using rustdoc. Just run
// `cargo doc --document-private-items --open` and look at the `echo` module.
aldrin::generate!(
    "src/echo.aldrin",
    introspection_if = "introspection",
    warnings_as_errors = true,
);

/// Echo example.
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
    /// Run an echo server.
    Server,

    /// List available echo servers.
    List,

    /// Send something to the server and have it echoed back.
    Echo(EchoArgs),

    /// Send something to the server and have it echoed back to all listeners.
    EchoAll(EchoArgs),

    /// Listen for events from a server.
    Listen(Listen),
}

#[derive(Parser)]
struct EchoArgs {
    /// UUID of the server to use.
    ///
    /// If this is not specified, then the first server is used that is found.
    #[clap(short, long)]
    server: Option<ObjectUuid>,

    /// String to send to the server.
    echo: String,
}

#[derive(Parser)]
struct Listen {
    /// UUID of the server to use.
    ///
    /// If this is not specified, then the first server is used that is found.
    #[clap(short, long)]
    server: Option<ObjectUuid>,
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
        echo::register_introspection_echo(&handle)?;
        handle.submit_introspection()?;
    }

    match args.cmd {
        Command::Server => server(&handle).await?,
        Command::List => list(&handle).await?,
        Command::Echo(args) => echo(&handle, args).await?,
        Command::EchoAll(args) => echo_all(&handle, args).await?,
        Command::Listen(args) => listen(&handle, args).await?,
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
    let mut echo = Echo::new(&object).await?;

    println!("Starting echo server {}.", object.id().uuid);

    loop {
        let call = tokio::select! {
            // Services are essentially asynchronous streams of incoming function calls. They also
            // implement the `Stream` trait from the `future` crate, which is however not used here.
            call = echo.next_call() => {
                match call {
                    Some(Ok(call)) => call,

                    // Function calls can be invalid either because their id is unknown or because
                    // the arguments to the call failed to deserialize.
                    Some(Err(e)) => {
                        println!("Received an invalid function call: {e}.");
                        continue;
                    }

                    // `None` is returned when the service is destroyed or when the connection to
                    // the broker shuts down.
                    None => return Err(anyhow!("broker shut down")),
                }
            }

            signal = signal::ctrl_c() => {
                signal.with_context(|| anyhow!("failed to listen for CTRL-C"))?;
                break Ok(());
            }
        };

        // `EchoCall` is a generated enum, that has one variant for each defined function. Call
        // arguments (if any) and a promise object are contained in the call object. Aldrin does not
        // impose any restrictions on when the reply is sent. If e.g. processing the call takes some
        // time, then it is fine to hang onto the call or the inner promise object until then. If
        // the promise object is dropped, then the caller will be notified that the call has been
        // aborted.
        match call {
            EchoCall::Echo(call) => {
                let (args, promise) = call.into_args_and_promise();
                println!("echo(\"{args}\") called.");

                if args.is_empty() {
                    promise.err(EchoEchoError::EmptyString)?;
                } else {
                    // Here, we echo the same value back to the caller.
                    promise.ok(args)?;
                }
            }

            EchoCall::EchoAll(call) => {
                let args = call.args();
                println!("echo_all(\"{args}\") called.");

                if args.is_empty() {
                    call.err(EchoEchoAllError::EmptyString)?;
                } else {
                    // This emits an event to all subscribed clients. If there is no such client,
                    // then the event will not even be sent to the broker. In any case, the event
                    // will be sent at most once. The broker then dispatches it further to other
                    // clients.
                    echo.echoed_to_all(args)?;

                    // No value is sent back to the caller.
                    call.done()?;
                }
            }
        }
    }
}

async fn list(bus: &Handle) -> Result<()> {
    // `Discoverer`s are used to find objects on the bus that implement some specific set of
    // services. Every call to `add_object` registers interest in one type of object.
    let mut discoverer = bus
        .create_discoverer()
        .any_object_with_services((), [EchoProxy::UUID])
        .build_current_only()
        .await?;

    let mut found = false;

    // The discoverer will emit events for every object it has found on the bus that matches the
    // criteria. Through the event, we can get the object's id and the relevant service ids.
    while let Some(event) = discoverer.next_event().await {
        println!("Found echo server {}.", event.object_id().uuid);
        found = true;
    }

    if !found {
        println!("No echo servers found.");
    }

    Ok(())
}

async fn echo(bus: &Handle, args: EchoArgs) -> Result<()> {
    let echo = get_echo(bus, args.server).await?;
    println!(
        "Calling echo(\"{}\") on {}.",
        args.echo,
        echo.id().object_id.uuid
    );

    // This is what a function call in Aldrin looks like on the client's side. The `?` handles
    // errors on the protocol level, such as when e.g. the server shuts down during any of these
    // calls. The `match` then operates on the reply from the echo server (`Result<String,
    // EchoEchoError>`).
    match echo.echo(args.echo).await?.into_args() {
        Ok(reply) => {
            println!("Server replied with: \"{reply}\".");
            Ok(())
        }

        Err(EchoEchoError::EmptyString) => Err(anyhow!("empty string")),
    }
}

async fn echo_all(bus: &Handle, args: EchoArgs) -> Result<()> {
    let echo = get_echo(bus, args.server).await?;
    println!(
        "Calling echo_all(\"{}\") on {}.",
        args.echo,
        echo.id().object_id.uuid
    );

    // This is just like `echo` above, except that the server returns no value. Instead, it will
    // emit an event with the value we sent. The event can be seen by having another instance of
    // this example running with the `listen` subcommand.
    match echo.echo_all(args.echo).await?.into_args() {
        Ok(()) => Ok(()),
        Err(EchoEchoAllError::EmptyString) => Err(anyhow!("empty string")),
    }
}

async fn listen(bus: &Handle, args: Listen) -> Result<()> {
    let mut echo = get_echo(bus, args.server).await?;

    // Events need to be subscribed to, or else the broker will not forward them to this client. In
    // fact, when there are no subscribers to an event at all, they will not even be sent to the
    // broker in the first place. As the name suggests, `subscribe_all` will subscribe to all events
    // of the Echo service.
    echo.subscribe_all().await?;

    println!("Listen to events from {}.", echo.id().object_id.uuid);

    loop {
        let event = tokio::select! {
            event = echo.next_event() => {
                match event {
                    Some(Ok(event)) => event,

                    // As with functions, events can be invalid, either because they have an unknown
                    // id or because their associated value failed to deserialize.
                    Some(Err(e)) => {
                        println!("Received an invalid event: {e}.");
                        continue;
                    }

                    // Happens when either the service is destroyed or the connection to the broker
                    // shuts down.
                    None => break Ok(()),
                }
            }

            signal = signal::ctrl_c() => {
                signal.with_context(|| anyhow!("failed to listen for CTRL-C"))?;
                break Ok(());
            }
        };

        // `EchoEvent` is a generated enum that has one variant for each event.
        match event {
            EchoEvent::EchoedToAll(event) => {
                println!("Received event: EchoedToAll(\"{}\").", event.args())
            }
        }
    }
}

async fn get_echo(bus: &Handle, object_uuid: Option<ObjectUuid>) -> Result<EchoProxy> {
    // Find either a specific object (when `object_uuid` is `Some`) or any object implementing the
    // Echo service. The `find_object_n` function is a convenience wrapper for `Discoverer`, which
    // is more concise when looking for a single object as a one-shot operation.
    let (_, [service_id]) = bus
        .find_object_n(object_uuid, &[EchoProxy::UUID])
        .await?
        .ok_or_else(|| anyhow!("echo server not found"))?;

    let echo = EchoProxy::new(bus, service_id).await?;
    Ok(echo)
}
