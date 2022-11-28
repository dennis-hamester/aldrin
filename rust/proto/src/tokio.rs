use crate::error::{DeserializeError, SerializeError};
use crate::message::{Message, MessageOps, Packetizer};
use crate::transport::AsyncTransport;
use bytes::{Buf, BytesMut};
use pin_project::pin_project;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

const INITIAL_CAPACITY: usize = 8 * 1024;
const BACKPRESSURE_BOUNDARY: usize = INITIAL_CAPACITY;

#[pin_project]
#[derive(Debug)]
pub struct TokioTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    #[pin]
    io: T,
    packetizer: Packetizer,
    write_buf: BytesMut,
}

impl<T> TokioTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    pub fn new(io: T) -> Self {
        TokioTransport {
            io,
            packetizer: Packetizer::new(),
            write_buf: BytesMut::with_capacity(INITIAL_CAPACITY),
        }
    }
}

impl<T> AsyncTransport for TokioTransport<T>
where
    T: AsyncRead + AsyncWrite,
{
    type Error = TokioTransportError;

    fn receive_poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Message, Self::Error>> {
        let mut this = self.project();

        loop {
            if let Some(buf) = this.packetizer.next_message() {
                return Poll::Ready(
                    Message::deserialize_message(buf).map_err(TokioTransportError::Deserialize),
                );
            }

            let mut read_buf = ReadBuf::uninit(this.packetizer.spare_capacity_mut());
            match this.io.as_mut().poll_read(cx, &mut read_buf) {
                Poll::Ready(Ok(())) if read_buf.filled().is_empty() => {
                    return Poll::Ready(Err(TokioTransportError::Io(
                        IoErrorKind::UnexpectedEof.into(),
                    )))
                }

                Poll::Ready(Ok(())) => {
                    // SAFETY: The first len bytes have been initialized.
                    let len = read_buf.filled().len();
                    unsafe {
                        this.packetizer.bytes_written(len);
                    }
                }

                Poll::Ready(Err(e)) => return Poll::Ready(Err(TokioTransportError::Io(e))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    fn send_poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        if self.write_buf.len() >= BACKPRESSURE_BOUNDARY {
            self.send_poll_flush(cx)
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        let this = self.project();

        let msg = msg
            .serialize_message()
            .map_err(TokioTransportError::Serialize)?;

        if this.write_buf.is_empty() {
            *this.write_buf = msg;
        } else {
            this.write_buf.extend_from_slice(&msg);
        }

        Ok(())
    }

    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        while !this.write_buf.is_empty() {
            match this.io.as_mut().poll_write(cx, this.write_buf) {
                Poll::Ready(Ok(0)) => {
                    return Poll::Ready(Err(TokioTransportError::Io(
                        IoErrorKind::WriteZero.into(),
                    )));
                }
                Poll::Ready(Ok(n)) => {
                    this.write_buf.advance(n);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Err(TokioTransportError::Io(e))),
                Poll::Pending => return Poll::Pending,
            }
        }

        this.io.poll_flush(cx).map_err(TokioTransportError::Io)
    }
}

#[derive(Debug)]
pub enum TokioTransportError {
    Io(IoError),
    Serialize(SerializeError),
    Deserialize(DeserializeError),
}

impl fmt::Display for TokioTransportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokioTransportError::Io(e) => e.fmt(f),
            TokioTransportError::Serialize(e) => e.fmt(f),
            TokioTransportError::Deserialize(e) => e.fmt(f),
        }
    }
}

impl StdError for TokioTransportError {}
