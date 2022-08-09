use super::BrokerRunArgs;
use crate::test::RunError;
use aldrin_codec::filter::Noop;
use aldrin_codec::packetizer::NewlineTerminated;
use aldrin_codec::serializer::Json;
use aldrin_codec::TokioCodec;
use aldrin_conformance_test_shared::broker::{FromBrokerMessage, FromBrokerReady, ToBrokerMessage};
use aldrin_proto::{
    AsyncTransport, AsyncTransportExt, ChannelCookie, ChannelEnd, Connect, ConnectReply,
    CreateChannel, DestroyChannelEnd, DestroyChannelEndResult, Message, VERSION,
};
use anyhow::{anyhow, Context, Error, Result};
use futures::future;
use futures::sink::{Sink, SinkExt};
use futures::stream::{StreamExt, TryStreamExt};
use std::net::Ipv4Addr;
use std::process::Stdio;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::{io, time};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

type StdinBox = Box<dyn Sink<ToBrokerMessage, Error = Error> + Unpin + Send + Sync>;
type TransportBox = Box<dyn AsyncTransport<Error = Error> + Unpin + Send + Sync>;

pub struct BrokerUnderTest {
    child: Child,
    stderr: Option<JoinHandle<Vec<u8>>>,
    stdin: Option<StdinBox>,
    port: u16,
    args: BrokerRunArgs,
}

impl BrokerUnderTest {
    pub async fn new(args: BrokerRunArgs) -> Result<Self, RunError> {
        let mut child = Command::new(&args.broker)
            .kill_on_drop(true)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| anyhow!("failed to spawn broker process"))
            .map_err(RunError::bare)?;

        let mut stdout = FramedRead::new(child.stdout.take().unwrap(), LinesCodec::new())
            .map_err(Error::from)
            .and_then(|line| future::ready(serde_json::from_str(&line).map_err(Error::from)));

        let mut stdin: Option<StdinBox> = Some(Box::new(
            FramedWrite::new(child.stdin.take().unwrap(), LinesCodec::new())
                .with(|msg| future::ready(serde_json::to_string(&msg).map_err(Error::from))),
        ));

        let mut stderr = child.stderr.take().unwrap();
        let mut stderr = Some(tokio::spawn(async move {
            let mut buf = Vec::new();
            io::copy(&mut stderr, &mut buf).await.ok();
            buf
        }));

        let msg = match stdout.next().await {
            Some(Ok(msg)) => Ok(msg),
            Some(Err(e)) => Err(e),
            None => Err(anyhow!("unexpected eof on broker's stdout")),
        };

        let FromBrokerMessage::Ready(FromBrokerReady { port }) = handle_error(
            msg,
            Duration::from_millis(args.shutdown_timeout),
            &mut child,
            &mut stderr,
            &mut stdin,
        )
        .await
        .map_err(|e| e.context("broker failed to signal ready state"))?;

        Ok(Self {
            child,
            stderr,
            stdin,
            port,
            args,
        })
    }

    pub async fn result(mut self, res: Result<(), impl Into<Error>>) -> Result<(), RunError> {
        handle_error(
            res,
            Duration::from_millis(self.args.shutdown_timeout),
            &mut self.child,
            &mut self.stderr,
            &mut self.stdin,
        )
        .await?;

        self.stdin.take();
        Ok(())
    }

    pub async fn connect_client(&mut self) -> Result<Client> {
        Client::connect(self.port).await
    }

    pub async fn connect_client_no_handshake(&mut self) -> Result<Client> {
        Client::connect_no_handshake(self.port).await
    }

    pub async fn send_stdin(&mut self, msg: ToBrokerMessage) -> Result<()> {
        let stdin = self.stdin.as_mut().unwrap();
        stdin.send(msg).await?;
        Ok(())
    }
}

pub struct Client {
    transport: TransportBox,
}

impl Client {
    async fn connect(port: u16) -> Result<Self> {
        let transport = Self::connect_and_handshake_impl(port).await?;
        Ok(Self { transport })
    }

    async fn connect_no_handshake(port: u16) -> Result<Self> {
        let transport = Self::connect_impl(port).await?;
        Ok(Self { transport })
    }

    async fn connect_impl(port: u16) -> Result<TransportBox> {
        let stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).await?;

        Ok(Box::new(
            TokioCodec::new(
                stream,
                NewlineTerminated::new(),
                Noop,
                Json::with_pretty(false),
            )
            .map_err(Error::from),
        ))
    }

    async fn connect_and_handshake_impl(port: u16) -> Result<TransportBox> {
        let mut transport = Self::connect_impl(port)
            .await
            .with_context(|| anyhow!("failed to connect to broker"))?;

        Self::handshake_impl(&mut transport)
            .await
            .with_context(|| anyhow!("handshake failed"))?;

        Ok(transport)
    }

    async fn handshake_impl(transport: &mut TransportBox) -> Result<()> {
        transport
            .send_and_flush(Message::Connect(Connect { version: VERSION }))
            .await
            .with_context(|| anyhow!("failed to send connect message to broker"))?;

        let msg = transport
            .receive()
            .await
            .with_context(|| anyhow!("failed to receive connect-reply message from broker"))?;

        match msg {
            Message::ConnectReply(ConnectReply::Ok) => Ok(()),
            Message::ConnectReply(ConnectReply::VersionMismatch(version)) => Err(anyhow!(
                "broker does not implement Aldrin protocol {}, but {}",
                VERSION,
                version
            )),
            _ => Err(anyhow!(
                "expected connect-reply message but received {}",
                serde_json::to_string(&msg).unwrap()
            )),
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<()> {
        self.transport.send_and_flush(msg).await?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Message> {
        let msg = self.transport.receive().await?;
        Ok(msg)
    }

    pub async fn create_channel(
        &mut self,
        serial: u32,
        claim: ChannelEnd,
    ) -> Result<ChannelCookie> {
        self.send(Message::CreateChannel(CreateChannel { serial, claim }))
            .await
            .with_context(|| anyhow!("failed to send create-channel message to broker"))?;

        let msg = self
            .receive()
            .await
            .with_context(|| anyhow!("failed to receive create-channel-reply message"))?;

        if let Message::CreateChannelReply(msg) = msg {
            if msg.serial != serial {
                Err(anyhow!(
                    "create-channel-reply received with serial {} but expected {}",
                    msg.serial,
                    serial
                ))
            } else if msg.cookie.0.is_nil() {
                Err(anyhow!(
                    "create-channel-reply received with nil channel cookie"
                ))
            } else {
                Ok(msg.cookie)
            }
        } else {
            Err(anyhow!(
                "expected create-channel-reply message but received {}",
                serde_json::to_string(&msg).unwrap()
            ))
        }
    }

    pub async fn send_destroy_channel_end(
        &mut self,
        serial: u32,
        cookie: ChannelCookie,
        end: ChannelEnd,
    ) -> Result<DestroyChannelEndResult> {
        self.send(Message::DestroyChannelEnd(DestroyChannelEnd {
            serial,
            cookie,
            end,
        }))
        .await
        .with_context(|| anyhow!("failed to send destroy-channel-end message to broker"))?;

        let msg = self
            .receive()
            .await
            .with_context(|| anyhow!("failed to receive destroy-channel-end-reply message"))?;

        if let Message::DestroyChannelEndReply(msg) = msg {
            if msg.serial == serial {
                Ok(msg.result)
            } else {
                Err(anyhow!(
                    "destroy-channel-end-reply received with serial {} but expected {}",
                    msg.serial,
                    serial
                ))
            }
        } else {
            Err(anyhow!(
                "expected destroy-channel-end-reply message but received {}",
                serde_json::to_string(&msg).unwrap()
            ))
        }
    }
}

async fn handle_error<T>(
    res: Result<T, impl Into<Error>>,
    timeout: Duration,
    child: &mut Child,
    stderr: &mut Option<JoinHandle<Vec<u8>>>,
    stdin: &mut Option<StdinBox>,
) -> Result<T, RunError> {
    let err = match res {
        Ok(t) => return Ok(t),
        Err(err) => err,
    };

    stdin.take();
    if time::timeout(timeout, child.wait()).await.is_err() {
        child.start_kill().ok();
    }

    let stderr = time::timeout(timeout, async {
        stderr.take().unwrap().await.unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    Err(RunError::with_stderr(err, stderr))
}
