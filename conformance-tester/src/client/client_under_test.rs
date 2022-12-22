use super::ClientRunArgs;
use crate::test::RunError;
use aldrin_conformance_test_shared::client::ToClientMessage;
use aldrin_proto::message::{Connect, ConnectReply, Message};
use aldrin_proto::tokio::TokioTransport;
use aldrin_proto::transport::{AsyncTransport, AsyncTransportExt};
use aldrin_proto::VERSION;
use anyhow::{anyhow, Context, Error, Result};
use futures::future;
use futures::sink::{Sink, SinkExt};
use std::net::Ipv4Addr;
use std::process::Stdio;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::{io, time};
use tokio_util::codec::{FramedWrite, LinesCodec};

type StdinBox = Box<dyn Sink<ToClientMessage, Error = Error> + Unpin + Send + Sync>;
type TransportBox = Box<dyn AsyncTransport<Error = Error> + Unpin + Send + Sync>;

pub struct ClientUnderTest {
    child: Child,
    transport: TransportBox,
    stderr: Option<JoinHandle<Vec<u8>>>,
    stdin: Option<StdinBox>,
    args: ClientRunArgs,
}

impl ClientUnderTest {
    pub async fn new(args: ClientRunArgs) -> Result<Self, RunError> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .with_context(|| anyhow!("failed to bind tcp listener"))
            .map_err(RunError::bare)?;

        let port = listener
            .local_addr()
            .with_context(|| anyhow!("failed to get local tcp listener address"))
            .map_err(RunError::bare)?
            .port();

        let mut child = Command::new(&args.client)
            .arg(port.to_string())
            .kill_on_drop(true)
            .stdout(Stdio::null())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| anyhow!("failed to spawn client process"))
            .map_err(RunError::bare)?;

        let mut stderr = child.stderr.take().unwrap();
        let mut stderr = Some(tokio::spawn(async move {
            let mut buf = Vec::new();
            io::copy(&mut stderr, &mut buf).await.ok();
            buf
        }));

        let mut stdin: Option<StdinBox> = Some(Box::new(
            FramedWrite::new(child.stdin.take().unwrap(), LinesCodec::new())
                .with(|msg| future::ready(serde_json::to_string(&msg).map_err(Error::from))),
        ));

        let stream = handle_error(
            listener.accept().await,
            Duration::from_millis(args.shutdown_timeout),
            &mut child,
            &mut stderr,
            &mut stdin,
        )
        .await
        .map_err(|e| e.context("client failed to connect"))?
        .0;

        let transport = Box::new(TokioTransport::new(stream).map_err(Error::from));

        Ok(ClientUnderTest {
            child,
            transport,
            stderr,
            stdin,
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

    pub async fn send_message(&mut self, msg: Message) -> Result<()> {
        self.transport.send_and_flush(msg).await?;
        Ok(())
    }

    pub async fn recv_message(&mut self) -> Result<Message> {
        let msg = self.transport.receive().await?;
        Ok(msg)
    }

    pub async fn connect(&mut self) -> Result<()> {
        let msg = self
            .recv_message()
            .await
            .with_context(|| anyhow!("failed to receive connect message"))?;

        match msg {
            Message::Connect(Connect {
                version: VERSION, ..
            }) => {}
            Message::Connect(Connect { version, .. }) => {
                self.send_message(Message::ConnectReply(ConnectReply::VersionMismatch(
                    VERSION,
                )))
                .await
                .ok();

                return Err(anyhow!(
                    "received connect with incorrect version {}",
                    version
                ));
            }
            _ => return Err(anyhow!("expected connect message but received {msg:?}")),
        }

        self.send_message(Message::ConnectReply(
            ConnectReply::ok_with_serialize_value(&())?,
        ))
        .await
        .with_context(|| anyhow!("failed to send connect-reply message"))?;

        Ok(())
    }

    pub async fn send_stdin(&mut self, msg: ToClientMessage) -> Result<()> {
        let stdin = self.stdin.as_mut().unwrap();
        stdin.send(msg).await?;
        Ok(())
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
