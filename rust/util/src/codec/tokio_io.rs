use super::{Packetizer, Serializer};
use aldrin_proto::{AsyncTransport, Message};
use bytes::BytesMut;
use pin_project::pin_project;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::io::ErrorKind as IoErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

const INITIAL_CAPACITY: usize = 8 * 1024;
const BACKPRESSURE_BOUNDARY: usize = INITIAL_CAPACITY;

#[pin_project]
#[derive(Debug)]
pub struct TokioCodec<T, P, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    S: Serializer,
{
    #[pin]
    io: T,
    packetizer: P,
    serializer: S,
    read_buf: BytesMut,
    write_buf: Option<BytesMut>,
    name: Option<String>,
}

impl<T, P, S> TokioCodec<T, P, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    S: Serializer,
{
    pub fn new(io: T, packetizer: P, serializer: S) -> Self {
        TokioCodec {
            io,
            packetizer,
            serializer,
            read_buf: BytesMut::with_capacity(INITIAL_CAPACITY),
            write_buf: Some(BytesMut::with_capacity(INITIAL_CAPACITY)),
            name: None,
        }
    }

    pub fn with_name<N>(io: T, packetizer: P, serializer: S, name: N) -> Self
    where
        N: Into<String>,
    {
        TokioCodec {
            io,
            packetizer,
            serializer,
            read_buf: BytesMut::with_capacity(INITIAL_CAPACITY),
            write_buf: Some(BytesMut::with_capacity(INITIAL_CAPACITY)),
            name: Some(name.into()),
        }
    }
}

impl<T, P, S> AsyncTransport for TokioCodec<T, P, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    S: Serializer,
{
    type Error = TokioCodecError<P::Error, S::Error>;

    fn receive_poll(
        self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Result<Option<Message>, Self::Error>> {
        let mut this = self.project();

        loop {
            match this.packetizer.decode(this.read_buf) {
                Ok(Some(pkt)) => match this.serializer.deserialize(pkt) {
                    Ok(msg) => return Poll::Ready(Ok(Some(msg))),
                    Err(e) => return Poll::Ready(Err(TokioCodecError::Serializer(e))),
                },
                Ok(None) => {}
                Err(e) => return Poll::Ready(Err(TokioCodecError::Packetizer(e))),
            }

            match this.io.as_mut().poll_read_buf(cx, this.read_buf) {
                Poll::Ready(Ok(0)) => return Poll::Ready(Ok(None)),
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(TokioCodecError::Io(e))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<bool, Self::Error>> {
        let write_buf = self.write_buf.as_ref().unwrap();

        if write_buf.len() >= BACKPRESSURE_BOUNDARY {
            self.send_poll_flush(cx).map_ok(|_| true)
        } else {
            Poll::Ready(Ok(true))
        }
    }

    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        let this = self.project();
        let write_buf = this.write_buf.as_mut().unwrap();

        let mut pkt = BytesMut::new();
        if let Err(e) = this.serializer.serialize(msg, &mut pkt) {
            return Err(TokioCodecError::Serializer(e));
        }

        if let Err(e) = this.packetizer.encode(pkt.freeze(), write_buf) {
            return Err(TokioCodecError::Packetizer(e));
        }

        Ok(())
    }

    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();
        let write_buf = this.write_buf.as_mut().unwrap();

        while !write_buf.is_empty() {
            match this.io.as_mut().poll_write_buf(cx, write_buf) {
                Poll::Ready(Ok(0)) => {
                    this.write_buf.take();
                    return Poll::Ready(Err(TokioCodecError::Io(IoErrorKind::WriteZero.into())));
                }
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(TokioCodecError::Io(e))),
                Poll::Pending => return Poll::Pending,
            }
        }

        this.io.poll_flush(cx).map_err(TokioCodecError::Io)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.write_buf.is_some() {
            match self.as_mut().send_poll_flush(cx) {
                Poll::Ready(Ok(())) => {}
                p => return p,
            }
        }

        let this = self.project();
        this.write_buf.take();
        this.io.poll_shutdown(cx).map_err(TokioCodecError::Io)
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug)]
pub enum TokioCodecError<P, S> {
    Io(IoError),
    Packetizer(P),
    Serializer(S),
}

impl<P, S> fmt::Display for TokioCodecError<P, S>
where
    P: fmt::Display,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokioCodecError::Io(e) => e.fmt(f),
            TokioCodecError::Packetizer(e) => e.fmt(f),
            TokioCodecError::Serializer(e) => e.fmt(f),
        }
    }
}

impl<P, S> StdError for TokioCodecError<P, S>
where
    P: fmt::Display + fmt::Debug,
    S: fmt::Display + fmt::Debug,
{
}
