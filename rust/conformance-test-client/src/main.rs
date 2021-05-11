use aldrin_client::{Client, Handle};
use aldrin_codec::filter::Noop;
use aldrin_codec::packetizer::NulTerminated;
use aldrin_codec::serializer::Json;
use aldrin_codec::TokioCodec;
use aldrin_conformance_test_shared::client::ToClientMessage;
use anyhow::{anyhow, Error, Result};
use futures::future::{self, TryFutureExt};
use futures::stream::{Stream, StreamExt, TryStreamExt};
use std::env;
use std::net::Ipv4Addr;
use tokio::io;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio_util::codec::{FramedRead, LinesCodec};

type StdinBox = Box<dyn Stream<Item = Result<ToClientMessage>> + Unpin>;

struct ClientUnderTest {
    client: Handle,
    join: JoinHandle<Result<()>>,
    stdin: StdinBox,
}

impl ClientUnderTest {
    async fn new(port: u16) -> Result<Self> {
        let stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).await?;
        let transport =
            TokioCodec::new(stream, NulTerminated::new(), Noop, Json::with_pretty(false));
        let client = Client::connect(transport).await?;
        let handle = client.handle().clone();
        let join = tokio::spawn(client.run().map_err(Error::from));

        let stdin = Box::new(
            FramedRead::new(io::stdin(), LinesCodec::new())
                .map_err(Error::from)
                .and_then(|line| future::ready(serde_json::from_str(&line).map_err(Error::from))),
        );

        Ok(Self {
            client: handle,
            join,
            stdin,
        })
    }

    async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                res = &mut self.join => {
                    res??;
                    return Ok(());
                }

                msg = self.stdin.next() => {
                    match msg {
                        Some(msg) => self.handle_stdin_message(msg?),
                        None => return Ok(()),
                    }
                }
            }
        }
    }

    fn handle_stdin_message(&self, msg: ToClientMessage) {
        match msg {
            ToClientMessage::Shutdown(()) => self.client.shutdown(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let port = env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("port not specified"))?
        .parse()?;

    let client = ClientUnderTest::new(port).await?;
    client.run().await?;
    Ok(())
}
