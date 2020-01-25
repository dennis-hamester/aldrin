// Copyright (c) 2019 Dennis Hamester <dennis.hamester@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use aldrin_proto::{Message, Transport};
use futures_channel::mpsc;
use futures_core::stream::Stream;
use futures_sink::Sink;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use futures_channel::mpsc::SendError;

pub fn channel(buffer: usize) -> (ConnectionTransport, ClientTransport) {
    let (conn_sink, client_stream) = mpsc::channel(buffer);
    let (client_sink, conn_stream) = mpsc::channel(buffer);

    (
        ConnectionTransport::new(None, conn_stream, conn_sink),
        ClientTransport::new(None, client_stream, client_sink),
    )
}

pub fn channel_with_name(buffer: usize, name: String) -> (ConnectionTransport, ClientTransport) {
    let (conn_sink, client_stream) = mpsc::channel(buffer);
    let (client_sink, conn_stream) = mpsc::channel(buffer);

    (
        ConnectionTransport::new(Some(name.clone()), conn_stream, conn_sink),
        ClientTransport::new(Some(name), client_stream, client_sink),
    )
}

#[derive(Debug)]
pub struct ConnectionTransport {
    name: Option<String>,
    stream: mpsc::Receiver<Message>,
    sink: mpsc::Sender<Message>,
}

impl ConnectionTransport {
    fn new(
        name: Option<String>,
        stream: mpsc::Receiver<Message>,
        sink: mpsc::Sender<Message>,
    ) -> Self {
        ConnectionTransport { name, stream, sink }
    }
}

impl Stream for ConnectionTransport {
    type Item = Result<Message, SendError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Message, SendError>>> {
        Pin::new(&mut self.stream)
            .poll_next(cx)
            .map(|msg| msg.map(Ok))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl Sink<Message> for ConnectionTransport {
    type Error = SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), SendError>> {
        Pin::new(&mut self.sink).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), SendError> {
        Pin::new(&mut self.sink).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), SendError>> {
        Pin::new(&mut self.sink).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), SendError>> {
        Pin::new(&mut self.sink).poll_close(cx)
    }
}

impl Transport for ConnectionTransport {
    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(String::as_str)
    }
}

#[derive(Debug)]
pub struct ClientTransport {
    name: Option<String>,
    stream: mpsc::Receiver<Message>,
    sink: mpsc::Sender<Message>,
}

impl ClientTransport {
    fn new(
        name: Option<String>,
        stream: mpsc::Receiver<Message>,
        sink: mpsc::Sender<Message>,
    ) -> Self {
        ClientTransport { name, stream, sink }
    }
}

impl Stream for ClientTransport {
    type Item = Result<Message, SendError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<Result<Message, SendError>>> {
        Pin::new(&mut self.stream)
            .poll_next(cx)
            .map(|msg| msg.map(Ok))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

impl Sink<Message> for ClientTransport {
    type Error = SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), SendError>> {
        Pin::new(&mut self.sink).poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), SendError> {
        Pin::new(&mut self.sink).start_send(item)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), SendError>> {
        Pin::new(&mut self.sink).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), SendError>> {
        Pin::new(&mut self.sink).poll_close(cx)
    }
}

impl Transport for ClientTransport {
    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(String::as_str)
    }
}
