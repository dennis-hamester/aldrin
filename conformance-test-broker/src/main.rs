use aldrin_broker::{Broker, BrokerHandle};
use aldrin_proto::tokio::TokioTransport;
use anyhow::{anyhow, Context, Error, Result};
use std::io::{self, Read};
use std::net::Ipv4Addr;
use std::thread;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot::{self, Receiver, Sender};
use tokio::task::JoinHandle;

struct BrokerUnderTest {
    broker: BrokerHandle,
    join: JoinHandle<()>,
    listener: TcpListener,
    stdin_closed: Receiver<Result<()>>,
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

        println!("{port}");

        let broker = Broker::new();
        let handle = broker.handle().clone();
        let join = tokio::spawn(broker.run());

        let (sender, stdin_closed) = oneshot::channel();
        thread::spawn(|| Self::wait_stdin_closed(sender));

        Ok(BrokerUnderTest {
            broker: handle,
            join,
            listener,
            stdin_closed,
        })
    }

    async fn run(mut self) -> Result<()> {
        loop {
            tokio::select! {
                res = &mut self.join => {
                    match res {
                        Ok(()) => break Err(anyhow!("broker shut down unexpectedly")),
                        Err(e) => {
                            break Err(
                                Error::new(e).context(anyhow!("broker shut down unexpectedly"))
                            );
                        }
                    }
                }

                stream = self.listener.accept() => {
                    let stream = stream
                        .with_context(|| anyhow!("failed to accept new connection"))?
                        .0;
                    tokio::spawn(Self::handle_new_connection(self.broker.clone(), stream));
                }

                res = &mut self.stdin_closed => {
                    let res = match res {
                        Ok(Ok(())) => Ok(()),
                        Ok(Err(e)) => Err(e),
                        Err(e) => Err(Error::new(e)
                            .context(anyhow!("thread reading from stdin shut down unexpectedly"))),
                    };

                    self.broker.shutdown().await;
                    self.join.await.ok();

                    break res;
                }
            }
        }
    }

    async fn handle_new_connection(mut broker: BrokerHandle, stream: TcpStream) -> Result<()> {
        let transport = TokioTransport::new(stream);

        let conn = broker
            .connect(transport)
            .await
            .with_context(|| anyhow!("failed to connect client"))?;

        conn.run()
            .await
            .with_context(|| anyhow!("connection closed unexpectedly"))
    }

    fn wait_stdin_closed(sender: Sender<Result<()>>) {
        let mut stdin = io::stdin().lock();
        let mut buf = [0; 64];

        loop {
            match stdin.read(&mut buf) {
                Ok(0) => {
                    sender.send(Ok(())).ok();
                    break;
                }

                Ok(_) => {}

                Err(e) => {
                    sender
                        .send(Err(
                            Error::new(e).context(anyhow!("failed to read from stdin"))
                        ))
                        .ok();
                    break;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let broker = BrokerUnderTest::new().await?;
    broker.run().await?;
    Ok(())
}
