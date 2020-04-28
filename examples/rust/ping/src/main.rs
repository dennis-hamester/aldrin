use aldrin_broker::{Broker, BrokerHandle};
use aldrin_client::{Client, ObjectUuid, SubscribeMode};
use aldrin_util::codec::{JsonSerializer, LengthPrefixed, TokioCodec};
use futures::future::select_all;
use futures::stream::StreamExt;
use std::error::Error;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use structopt::clap::AppSettings;
use structopt::StructOpt;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::time::{delay_for, Duration};
use uuid::Uuid;

aldrin_codegen_macros::generate!("../../schemas/ping.aldrin");

const MEASURE_UPDATE_MS: u64 = 5000;

#[derive(StructOpt)]
#[structopt(no_version)]
struct BrokerArgs {
    /// IP address and port
    #[structopt(short, long, default_value = "127.0.0.1:5000")]
    listen: SocketAddr,

    /// Internal broker fifo size
    ///
    /// The default is defined by the broker implementation. Use 0 to make the fifo unbounded.
    #[structopt(short, long)]
    broker_fifo_size: Option<usize>,

    /// Internal connection fifo size
    ///
    /// The default is defined by the broker implementation. Use 0 to make the fifo unbounded.
    #[structopt(short, long)]
    conn_fifo_size: Option<usize>,
}

async fn add_connection(
    socket: TcpStream,
    addr: SocketAddr,
    mut handle: BrokerHandle,
    fifo_size: Option<usize>,
) -> Result<(), Box<dyn Error>> {
    println!("Incoming connection from {}.", addr);

    let t = TokioCodec::new(socket, LengthPrefixed::new(), JsonSerializer::new(true));
    let conn = if let Some(fifo_size) = fifo_size {
        handle
            .add_connection_with_fifo_size(t, NonZeroUsize::new(fifo_size))
            .await?
    } else {
        handle.add_connection(t).await?
    };
    println!("Connection from {} established.", addr);

    conn.run().await?;
    println!("Connection from {} closed.", addr);

    Ok(())
}

async fn broker(args: BrokerArgs) -> Result<(), Box<dyn Error>> {
    let broker = if let Some(fifo_size) = args.broker_fifo_size {
        Broker::with_fifo_size(NonZeroUsize::new(fifo_size))
    } else {
        Broker::new()
    };
    let handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut listener = TcpListener::bind(&args.listen).await?;
    println!("Listen for connections on {}.", args.listen);

    loop {
        let (socket, addr) = listener.accept().await?;
        let handle = handle.clone();
        let conn_fifo_size = args.conn_fifo_size;
        tokio::spawn(async move {
            if let Err(e) = add_connection(socket, addr, handle, conn_fifo_size).await {
                println!("Error on connection from {}: {}.", addr, e);
            }
        });
    }
}

#[derive(StructOpt)]
#[structopt(no_version)]
struct RunArgs {
    /// IP address and port of the broker
    #[structopt(short, long, default_value = "127.0.0.1:5000")]
    broker: SocketAddr,

    /// Internal client fifo size
    #[structopt(short, long, default_value = "16")]
    fifo_size: usize,

    /// Delay in milliseconds between pings
    #[structopt(short, long, default_value = "1000")]
    delay: u32,
}

async fn run(args: RunArgs) -> Result<(), Box<dyn Error>> {
    let addr = args.broker;
    println!("Connecting to broker at {}.", addr);

    let socket = TcpStream::connect(&addr).await?;
    let t = TokioCodec::new(socket, LengthPrefixed::new(), JsonSerializer::new(true));
    let client = Client::connect(t, args.fifo_size).await?;
    let mut handle = client.handle().clone();
    tokio::spawn(client.run());
    println!("Connection to broker at {} established.", addr);

    let mut obj = handle.create_object(ObjectUuid(Uuid::new_v4())).await?;
    let ping = ping::Ping::create(&mut obj, args.fifo_size).await?;
    let mut emitter = ping.event_emitter().unwrap();
    let mut sc = handle
        .services_created(SubscribeMode::All, args.fifo_size)
        .await?;
    let mut others = Vec::new();
    let mut delay = delay_for(Duration::from_millis(args.delay as u64));
    let mut measure = delay_for(Duration::from_millis(MEASURE_UPDATE_MS));
    let mut outgoing = 0;
    let mut incoming = 0;

    loop {
        let poll_events = !others.is_empty();
        let events = async { select_all(others.iter_mut().map(StreamExt::next)).await };

        select! {
            Some(id) = sc.next() => {
                if id.uuid == ping::PING_UUID {
                    let other = ping::PingProxy::bind(handle.clone(), id)?;
                    let mut events = other.events(args.fifo_size);
                    events.subscribe_ping().await?;
                    others.push(events);
                }
            }

            _ = &mut delay => {
                delay = delay_for(Duration::from_millis(args.delay as u64));
                emitter.ping().await?;
                outgoing += 1;
            }

            (ev, idx, _) = events, if poll_events => {
                match ev {
                    Some(_) => { incoming += 1; }
                    None => { others.remove(idx); }
                }
            }

            _ = &mut measure => {
                measure = delay_for(Duration::from_millis(MEASURE_UPDATE_MS));
                println!();
                println!("Statistics over the last {} milliseconds:", MEASURE_UPDATE_MS);
                println!("Outgoing pings: {}", outgoing);
                println!("Incoming pings: {}", incoming);
                outgoing = 0;
                incoming = 0;
            }
        }
    }
}

#[derive(StructOpt)]
#[structopt(
    global_settings = &[
        AppSettings::VersionlessSubcommands,
        AppSettings::ColoredHelp,
        AppSettings::DisableVersion,
    ],
    no_version,
)]
enum Args {
    /// Runs an Aldrin broker
    Broker(BrokerArgs),

    /// Runs the ping example
    Run(RunArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match Args::from_args() {
        Args::Broker(args) => broker(args).await?,
        Args::Run(args) => run(args).await?,
    };

    Ok(())
}
