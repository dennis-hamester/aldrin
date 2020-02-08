use super::{Packetizer, Serializer};
use aldrin_proto::{Message, Transport};
use bytes::BytesMut;
use futures_core::stream::Stream;
use futures_sink::Sink;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

const INITIAL_CAPACITY: usize = 8 * 1024;
const BACKPRESSURE_BOUNDARY: usize = INITIAL_CAPACITY;

#[derive(Debug)]
pub struct TokioCodec<T, P, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    S: Serializer,
{
    io: T,
    packetizer: P,
    serializer: S,
    read_buf: BytesMut,
    write_buf: BytesMut,
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
            write_buf: BytesMut::with_capacity(INITIAL_CAPACITY),
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
            write_buf: BytesMut::with_capacity(INITIAL_CAPACITY),
            name: Some(name.into()),
        }
    }
}

impl<T, P, S> Stream for TokioCodec<T, P, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    S: Serializer,
{
    type Item = Result<Message, TokioCodecError<P::Error, S::Error>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = unsafe { Pin::into_inner_unchecked(self) };

        loop {
            match this.packetizer.decode(&mut this.read_buf) {
                Ok(Some(pkt)) => match this.serializer.deserialize(pkt) {
                    Ok(msg) => return Poll::Ready(Some(Ok(msg))),
                    Err(e) => return Poll::Ready(Some(Err(TokioCodecError::Serializer(e)))),
                },
                Ok(None) => {}
                Err(e) => return Poll::Ready(Some(Err(TokioCodecError::Packetizer(e)))),
            }

            let io = unsafe { Pin::new_unchecked(&mut this.io) };
            match io.poll_read_buf(cx, &mut this.read_buf) {
                Poll::Ready(Ok(0)) => return Poll::Ready(None),
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(TokioCodecError::Io(e)))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<T, P, S> Sink<Message> for TokioCodec<T, P, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    S: Serializer,
{
    type Error = TokioCodecError<P::Error, S::Error>;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let this = unsafe { Pin::into_inner_unchecked(self.as_mut()) };

        if this.write_buf.len() >= BACKPRESSURE_BOUNDARY {
            self.poll_flush(cx)
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        let this = unsafe { Pin::into_inner_unchecked(self) };

        let mut pkt = BytesMut::new();
        if let Err(e) = this.serializer.serialize(item, &mut pkt) {
            return Err(TokioCodecError::Serializer(e));
        }

        if let Err(e) = this.packetizer.encode(pkt.freeze(), &mut this.write_buf) {
            return Err(TokioCodecError::Packetizer(e));
        }

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let this = unsafe { Pin::into_inner_unchecked(self) };

        while !this.write_buf.is_empty() {
            let io = unsafe { Pin::new_unchecked(&mut this.io) };
            match io.poll_write_buf(cx, &mut this.write_buf) {
                Poll::Ready(Ok(_)) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(TokioCodecError::Io(e))),
                Poll::Pending => return Poll::Pending,
            }
        }

        let io = unsafe { Pin::new_unchecked(&mut this.io) };
        io.poll_flush(cx).map_err(TokioCodecError::Io)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self.as_mut().poll_flush(cx) {
            Poll::Ready(Ok(())) => {}
            p => return p,
        }

        let this: &mut Self = unsafe { Pin::into_inner_unchecked(self) };
        let io = unsafe { Pin::new_unchecked(&mut this.io) };
        io.poll_shutdown(cx).map_err(TokioCodecError::Io)
    }
}

impl<T, P, S> Transport for TokioCodec<T, P, S>
where
    T: AsyncRead + AsyncWrite,
    P: Packetizer,
    S: Serializer,
{
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
