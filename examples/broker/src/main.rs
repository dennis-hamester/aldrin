use aldrin_broker::core::tokio::TokioTransport;
use aldrin_broker::{Broker, BrokerHandle};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::time;

const BIND_DEFAULT: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9999);
const STATISTICS_INTERVAL: Duration = Duration::from_secs(60);

/// Aldrin broker for the examples.
#[derive(Parser)]
struct Args {
    /// Address to bind the broker's TCP socket to.
    #[clap(default_value_t = BIND_DEFAULT)]
    bind: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Starting broker on {}.", args.bind);

    // The Broker created further below does by itself not deal with listening for new
    // connections. The application that hosts the broker has to do that via some means. Here, we
    // use Tokio's `TcpListener` to accept connections via TCP.
    let listener = TcpListener::bind(&args.bind)
        .await
        .with_context(|| anyhow!("failed to bind to {}", args.bind))?;

    // Create a broker and save a handle to it. The handle is later used to add connections and to
    // shut down the broker. The broker itself must be run explicitly.
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());
    let mut statistics = time::interval(STATISTICS_INTERVAL);

    loop {
        // Accept new connections, but also enable clean shut downs when pressing CTRL-C.
        let new_conn = tokio::select! {
            new_conn = listener.accept() => new_conn,

            signal = signal::ctrl_c() => {
                signal.with_context(|| anyhow!("failed to listen for CTRL-C"))?;
                break;
            }

            _ = statistics.tick() => {
                #[cfg(feature = "statistics")]
                {
                    let statistics = handle.take_statistics().await?;
                    let duration = (statistics.end() - statistics.start()).as_secs();

                    println!();
                    println!("Statistics for the last {} second(s):", duration);
                    println!(" - Messages sent: {}", statistics.messages_sent());
                    println!(" - Messages received: {}", statistics.messages_received());
                    println!(" - Connections: {}", statistics.num_connections());
                    println!(" - Objects: {}", statistics.num_objects());
                    println!(" - Services: {}", statistics.num_services());
                    println!(" - Channels: {}", statistics.num_channels());
                    println!(" - Bus listeners: {}", statistics.num_bus_listeners());
                    println!();
                }

                continue;
            }
        };

        let (stream, addr) = match new_conn {
            Ok(new_conn) => new_conn,

            Err(e) => {
                println!("Failed to accept new connection: {}.", e);
                continue;
            }
        };

        // New connections are handled in a new task, so as to not block this loop.
        println!("New connection from {}.", addr);
        let handle = handle.clone();
        tokio::spawn(async move {
            match handle_connection(handle, stream).await {
                Ok(()) => println!("Connection from {} shut down.", addr),
                Err(e) => println!("Error on connection from {}: {:#}", addr, e),
            }
        });
    }

    // The broker can be shut down cleanly. This will notify all clients as well. The broker's task
    // will join once all connections have been shut down.
    println!("Shutting down broker.");
    handle.shutdown().await;
    join.await
        .with_context(|| anyhow!("failed to join broker task"))?;

    Ok(())
}

async fn handle_connection(mut handle: BrokerHandle, stream: TcpStream) -> Result<()> {
    // Aldrin uses so-called "transports" to abstract from connection details, such as TCP
    // sockets. Transports are defined by the `AsyncTransport` trait in the `aldrin-core` crate.
    //
    // Here, we wrap a `TcpStream` from Tokio in a `TokioTransport`, which then implements
    // `AsyncTransport`. Any type that implements Tokio's `AsyncRead` and `AsyncWrite` traits can be
    // used like that.
    let transport = TokioTransport::new(stream);

    // Transports are added to the broker through the handle, which then performs the initial
    // handshake with the client.
    //
    // This example uses the simple `connect` function, which does the entire handshake in one
    // step. Aldrin however also allows passing custom data between broker and client during this
    // phase. This can be done using the handle's `begin_connect` function.
    let conn = handle
        .connect(transport)
        .await
        .with_context(|| anyhow!("failed to connect transport"))?;

    // The result of connecting a transport is a connection, which must be run just like the broker
    // itself. Connections also have handles (`conn.handle()`), which can be used to shut down
    // individual connections.
    conn.run()
        .await
        .with_context(|| anyhow!("failed to run connection"))
}
