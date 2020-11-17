use super::{Filter, Packetizer, Serializer};
use aldrin_proto::{AsyncTransport, Message};
use bytes::BytesMut;
use pin_project::pin_project;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

const INITIAL_CAPACITY: usize = 8 * 1024;
const BACKPRESSURE_BOUNDARY: usize = INITIAL_CAPACITY;

#[pin_project]
#[derive(Debug)]
pub struct TokioCodec<T, P, F, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    F: Filter,
    S: Serializer,
{
    #[pin]
    io: T,
    packetizer: P,
    filter: F,
    serializer: S,
    read_buf: BytesMut,
    write_buf: Option<BytesMut>,
    name: Option<String>,
}

impl<T, P, F, S> TokioCodec<T, P, F, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    F: Filter,
    S: Serializer,
{
    pub fn new(io: T, packetizer: P, filter: F, serializer: S) -> Self {
        TokioCodec {
            io,
            packetizer,
            filter,
            serializer,
            read_buf: BytesMut::with_capacity(INITIAL_CAPACITY),
            write_buf: Some(BytesMut::with_capacity(INITIAL_CAPACITY)),
            name: None,
        }
    }

    pub fn with_name<N>(io: T, packetizer: P, filter: F, serializer: S, name: N) -> Self
    where
        N: Into<String>,
    {
        TokioCodec {
            io,
            packetizer,
            filter,
            serializer,
            read_buf: BytesMut::with_capacity(INITIAL_CAPACITY),
            write_buf: Some(BytesMut::with_capacity(INITIAL_CAPACITY)),
            name: Some(name.into()),
        }
    }
}

impl<T, P, F, S> AsyncTransport for TokioCodec<T, P, F, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    F: Filter,
    S: Serializer,
{
    type Error = TokioCodecError<P::Error, F::Error, S::Error>;

    fn receive_poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Message, Self::Error>> {
        let mut this = self.project();

        loop {
            match this.packetizer.decode(this.read_buf) {
                Ok(Some(pkt)) => match this.filter.backward(pkt) {
                    Ok(pkt) => match this.serializer.deserialize(pkt.freeze()) {
                        Ok(msg) => return Poll::Ready(Ok(msg)),
                        Err(e) => return Poll::Ready(Err(TokioCodecError::Serializer(e))),
                    },
                    Err(e) => return Poll::Ready(Err(TokioCodecError::Filter(e))),
                },
                Ok(None) => {}
                Err(e) => return Poll::Ready(Err(TokioCodecError::Packetizer(e))),
            }

            match this.io.as_mut().poll_read_buf(cx, this.read_buf) {
                Poll::Ready(Ok(0)) => {
                    return Poll::Ready(Err(TokioCodecError::Io(IoErrorKind::UnexpectedEof.into())))
                }
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(TokioCodecError::Io(e))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let write_buf = self.write_buf.as_ref().unwrap();

        if write_buf.len() >= BACKPRESSURE_BOUNDARY {
            self.send_poll_flush(cx)
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        let this = self.project();
        let write_buf = this.write_buf.as_mut().unwrap();

        let pkt = this
            .serializer
            .serialize(msg)
            .map_err(TokioCodecError::Serializer)?;

        let pkt = this.filter.forward(pkt).map_err(TokioCodecError::Filter)?;

        this.packetizer
            .encode(pkt.freeze(), write_buf)
            .map_err(TokioCodecError::Packetizer)?;

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

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

#[derive(Debug)]
pub enum TokioCodecError<P, F, S> {
    Io(IoError),
    Packetizer(P),
    Filter(F),
    Serializer(S),
}

impl<P, F, S> fmt::Display for TokioCodecError<P, F, S>
where
    P: fmt::Display,
    F: fmt::Display,
    S: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokioCodecError::Io(e) => e.fmt(f),
            TokioCodecError::Packetizer(e) => e.fmt(f),
            TokioCodecError::Filter(e) => e.fmt(f),
            TokioCodecError::Serializer(e) => e.fmt(f),
        }
    }
}

impl<P, F, S> StdError for TokioCodecError<P, F, S>
where
    P: fmt::Display + fmt::Debug,
    F: fmt::Display + fmt::Debug,
    S: fmt::Display + fmt::Debug,
{
}
