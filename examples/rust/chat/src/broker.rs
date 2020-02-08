use super::{BrokerArgs, FIFO_SIZE};
use aldrin_broker::{Broker, BrokerHandle};
use aldrin_util::codec::{JsonSerializer, LengthPrefixed, TokioCodec};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

async fn add_connection(
    socket: TcpStream,
    addr: SocketAddr,
    mut handle: BrokerHandle,
) -> Result<(), Box<dyn Error>> {
    println!("Incoming connection from {}.", addr);

    let t = TokioCodec::new(socket, LengthPrefixed::new(), JsonSerializer::new(true));
    let conn = handle
        .add_connection::<_, Box<dyn Error>>(t, FIFO_SIZE)
        .await?;
    println!("Connection from {} established.", addr);

    conn.run::<Box<dyn Error>>().await?;
    println!("Connection from {} closed.", addr);

    Ok(())
}

pub(crate) async fn run(args: BrokerArgs) -> Result<(), Box<dyn Error>> {
    let broker = Broker::new(FIFO_SIZE);
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
