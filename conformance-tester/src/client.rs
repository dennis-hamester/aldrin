use crate::util::FutureExt;
use aldrin_proto::message::Message;
use aldrin_proto::tokio::{TokioTransport, TokioTransportError};
use aldrin_proto::transport::{AsyncTransport, AsyncTransportExt};
use anyhow::{anyhow, Context, Error, Result};
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpStream;
use tokio::time::Instant;

type TransportBox = Box<dyn AsyncTransport<Error = TokioTransportError> + Unpin + Send + Sync>;

pub struct Client {
    transport: TransportBox,
    sync: bool,
    shutdown: bool,
}

impl Client {
    pub async fn connect(port: u16, timeout: Instant, sync: bool, shutdown: bool) -> Result<Self> {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

        let stream = TcpStream::connect(addr)
            .timeout_at(timeout)
            .await
            .map_err(|_| anyhow!("timeout while connecting to broker at {}", addr))?
            .with_context(|| anyhow!("failed to connect to broker at {}", addr))?;

        let transport = Box::new(TokioTransport::new(stream));
        Ok(Self {
            transport,
            sync,
            shutdown,
        })
    }

    pub fn sync(&self) -> bool {
        self.sync
    }

    pub fn shutdown(&self) -> bool {
        self.shutdown
    }

    pub async fn send(&mut self, msg: Message) -> Result<()> {
        self.transport
            .send_and_flush(msg)
            .await
            .map_err(Error::from)
    }

    pub async fn receive(&mut self) -> Result<Message> {
        self.transport.receive().await.map_err(Error::from)
    }

    pub async fn expect_closed(&mut self) -> Result<(), Result<Message>> {
        match self.transport.receive().await {
            Ok(msg) => Err(Ok(msg)),
            Err(TokioTransportError::Io(e)) if e.kind() == ErrorKind::UnexpectedEof => Ok(()),
            Err(e) => Err(Err(Error::from(e))),
        }
    }
}
