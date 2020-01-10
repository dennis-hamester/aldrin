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

use aldrin_proto::Message;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ConnectionError {
    InternalError,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionError::InternalError => f.write_str("internal error"),
        }
    }
}

impl Error for ConnectionError {}

#[derive(Debug, Clone)]
pub enum EstablishError {
    InternalError,
    UnexpectedClientShutdown,
    UnexpectedMessageReceived(Message),
    VersionMismatch(u32),
    BrokerShutdown,
}

impl fmt::Display for EstablishError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EstablishError::InternalError => f.write_str("internal error"),
            EstablishError::UnexpectedClientShutdown => f.write_str("unexpected client shutdown"),
            EstablishError::UnexpectedMessageReceived(_) => {
                f.write_str("unexpected message received")
            }
            EstablishError::VersionMismatch(v) => {
                f.write_fmt(format_args!("client version {} mismatch", v))
            }
            EstablishError::BrokerShutdown => f.write_str("broker shutdown"),
        }
    }
}

impl Error for EstablishError {}
