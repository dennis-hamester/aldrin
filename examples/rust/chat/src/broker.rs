// Copyright (c) 2020 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

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
