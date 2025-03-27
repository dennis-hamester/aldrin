use crate::util::FutureExt;
use aldrin_core::message::{Message, MessageOps};
use aldrin_core::tokio::{TokioTransport, TokioTransportError};
use aldrin_core::transport::{AsyncTransport, AsyncTransportExt};
use aldrin_core::ProtocolVersion;
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
    version: ProtocolVersion,
}

impl Client {
    pub async fn connect(
        port: u16,
        timeout: Instant,
        sync: bool,
        shutdown: bool,
        version: ProtocolVersion,
    ) -> Result<Self> {
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
            version,
        })
    }

    pub fn sync(&self) -> bool {
        self.sync
    }

    pub fn shutdown(&self) -> bool {
        self.shutdown
    }

    pub async fn send(&mut self, mut msg: Message) -> Result<()> {
        msg.convert_value(None, self.version)?;

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
