use super::AsyncTransport;
use crate::message::Message;
use pin_project_lite::pin_project;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
    #[derive(Debug)]
    pub struct Buffered<T> {
        #[pin]
        inner: T,

        buffer: VecDeque<Message>,
    }
}

impl<T> Buffered<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            buffer: VecDeque::new(),
        }
    }
}

impl<T: AsyncTransport> AsyncTransport for Buffered<T> {
    type Error = T::Error;

    fn receive_poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<Message, Self::Error>> {
        self.project().inner.receive_poll(cx)
    }

    fn send_poll_ready(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn send_start(self: Pin<&mut Self>, msg: Message) -> Result<(), Self::Error> {
        self.project().buffer.push_back(msg);
        Ok(())
    }

    fn send_poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        while !this.buffer.is_empty() {
            match this.inner.as_mut().send_poll_ready(cx) {
                Poll::Ready(Ok(())) => {
                    let msg = this.buffer.pop_front().unwrap();
                    this.inner.as_mut().send_start(msg)?;
                }

                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }

        this.inner.as_mut().send_poll_flush(cx)
    }
}
