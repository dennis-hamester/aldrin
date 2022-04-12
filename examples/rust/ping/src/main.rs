use aldrin_broker::{Broker, BrokerHandle};
use aldrin_client::{Client, ObjectUuid, ServiceEvent, SubscribeMode};
use aldrin_codec::filter::Noop;
use aldrin_codec::packetizer::LengthPrefixed;
use aldrin_codec::serializer::Json;
use aldrin_codec::TokioCodec;
use anyhow::Result;
use clap::Parser;
use futures::future::select_all;
use futures::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::time::{sleep, Duration};

aldrin_client::generate!("../../schemas/ping.aldrin", warnings_as_errors = true);

const MEASURE_UPDATE_MS: u64 = 5000;

#[derive(Parser)]
struct BrokerArgs {
    /// IP address and port
    #[clap(short, long, default_value = "127.0.0.1:5000")]
    listen: SocketAddr,
}

async fn add_connection(
    socket: TcpStream,
    addr: SocketAddr,
    mut handle: BrokerHandle,
) -> Result<()> {
    println!("Incoming connection from {}.", addr);

    let t = TokioCodec::new(socket, LengthPrefixed::default(), Noop, Json::default());
    let conn = handle.add_connection(t).await?;
    println!("Connection from {} established.", addr);

    conn.run().await?;
    println!("Connection from {} closed.", addr);

    Ok(())
}

async fn broker(args: BrokerArgs) -> Result<()> {
    let broker = Broker::new();
    let handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let listener = TcpListener::bind(&args.listen).await?;
    println!("Listen for connections on {}.", args.listen);

    loop {
        let (socket, addr) = listener.accept().await?;
        let handle = handle.clone();
        tokio::spawn(async move {
            if let Err(e) = add_connection(socket, addr, handle).await {
                println!("Error on connection from {}: {}.", addr, e);
            }
        });
    }
}

#[derive(Parser)]
struct RunArgs {
    /// IP address and port of the broker
    #[clap(short, long, default_value = "127.0.0.1:5000")]
    broker: SocketAddr,

    /// Delay in milliseconds between pings
    #[clap(short, long, default_value = "1000")]
    delay: u32,
}

async fn run(args: RunArgs) -> Result<()> {
    let addr = args.broker;
    println!("Connecting to broker at {}.", addr);

    let socket = TcpStream::connect(&addr).await?;
    let t = TokioCodec::new(socket, LengthPrefixed::default(), Noop, Json::default());
    let client = Client::connect(t).await?;
    let handle = client.handle().clone();
    tokio::spawn(client.run());
    println!("Connection to broker at {} established.", addr);

    let obj = handle.create_object(ObjectUuid::new_v4()).await?;
    let ping = ping::Ping::create(&obj).await?;
    let emitter = ping.event_emitter();
    let mut svcs = handle.services(SubscribeMode::All)?;
    let mut others = Vec::new();
    let mut delay = Box::pin(sleep(Duration::from_millis(args.delay as u64)));
    let mut measure = Box::pin(sleep(Duration::from_millis(MEASURE_UPDATE_MS)));
    let mut outgoing = 0;
    let mut incoming = 0;

    loop {
        let poll_events = !others.is_empty();
        let events = async { select_all(others.iter_mut().map(StreamExt::next)).await };

        select! {
            Some(ServiceEvent::Created(id)) = svcs.next() => {
                if id.uuid == ping::PING_UUID {
                    let other = ping::PingProxy::bind(handle.clone(), id).await?;
                    let mut events = other.events();
                    events.subscribe_ping().await?;
                    others.push(events);
                }
            }

            _ = &mut delay => {
                delay = Box::pin(sleep(Duration::from_millis(args.delay as u64)));
                emitter.ping()?;
                outgoing += 1;
            }

            (ev, idx, _) = events, if poll_events => {
                match ev {
                    Some(_) => { incoming += 1; }
                    None => { others.remove(idx); }
                }
            }

            _ = &mut measure => {
                measure = Box::pin(sleep(Duration::from_millis(MEASURE_UPDATE_MS)));
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

#[derive(Parser)]
enum Args {
    /// Runs an Aldrin broker
    Broker(BrokerArgs),

    /// Runs the ping example
    Run(RunArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    match Args::parse() {
        Args::Broker(args) => broker(args).await,
        Args::Run(args) => run(args).await,
    }
}
