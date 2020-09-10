use super::BrokerArgs;
use aldrin_broker::{Broker, BrokerHandle};
use aldrin_codec::filter::Noop;
use aldrin_codec::packetizer::LengthPrefixed;
use aldrin_codec::serializer::Json;
use aldrin_codec::TokioCodec;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

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

pub(crate) async fn run(args: BrokerArgs) -> Result<()> {
    let broker = Broker::new();
    let handle = broker.handle().clone();
    tokio::spawn(broker.run());

    let mut listener = TcpListener::bind(&args.bind).await?;
    println!("Listen for connections on {}.", args.bind);

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
