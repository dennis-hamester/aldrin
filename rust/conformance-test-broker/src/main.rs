use aldrin_broker::{Broker, BrokerHandle};
use aldrin_codec::filter::Noop;
use aldrin_codec::packetizer::NewlineTerminated;
use aldrin_codec::serializer::Json;
use aldrin_codec::TokioCodec;
use aldrin_conformance_test_shared::broker::{FromBrokerMessage, FromBrokerReady, ToBrokerMessage};
use anyhow::{anyhow, Context, Error, Result};
use futures::future;
use futures::stream::{Stream, StreamExt, TryStreamExt};
use std::net::Ipv4Addr;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_util::codec::{FramedRead, LinesCodec};

type StdinBox = Box<dyn Stream<Item = Result<ToBrokerMessage>> + Unpin>;

struct BrokerUnderTest {
    broker: BrokerHandle,
    join: JoinHandle<()>,
    listener: TcpListener,
    stdin: StdinBox,
}

impl BrokerUnderTest {
    async fn new() -> Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .with_context(|| anyhow!("failed to bind tcp listener"))?;

        let port = listener
            .local_addr()
            .with_context(|| anyhow!("failed to get local tcp listener address"))?
            .port();

        println!(
            "{}",
            serde_json::to_string(&FromBrokerMessage::Ready(FromBrokerReady { port })).unwrap()
        );

        let broker = Broker::new();
        let handle = broker.handle().clone();
        let join = tokio::spawn(broker.run());

        let stdin = Box::new(
            FramedRead::new(io::stdin(), LinesCodec::new())
                .map_err(Error::from)
                .and_then(|line| future::ready(serde_json::from_str(&line).map_err(Error::from))),
        );

        Ok(BrokerUnderTest {
            broker: handle,
            join,
            listener,
            stdin,
        })
    }

    async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                res = &mut self.join => {
                    return res.with_context(|| anyhow!("failed to join broker task"));
                }

                stream = self.listener.accept() => {
                    let stream = stream
                        .with_context(|| anyhow!("failed to accept new connection"))?
                        .0;
                    tokio::spawn(Self::handle_new_connection(self.broker.clone(), stream));
                }

                msg = self.stdin.next() => {
                    match msg {
                        Some(msg) => self.handle_stdin_message(msg?).await,
                        None => return Ok(()),
                    }
                }
            }
        }
    }

    async fn handle_new_connection(mut broker: BrokerHandle, stream: TcpStream) -> Result<()> {
        let transport = TokioCodec::new(
            stream,
            NewlineTerminated::new(),
            Noop,
            Json::with_pretty(false),
        );
        let conn = broker.add_connection(transport).await?;
        tokio::spawn(conn.run());
        Ok(())
    }

    async fn handle_stdin_message(&mut self, msg: ToBrokerMessage) {
        match msg {
            ToBrokerMessage::Shutdown(()) => self.broker.shutdown().await,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let broker = BrokerUnderTest::new().await?;
    broker.run().await?;
    Ok(())
}
