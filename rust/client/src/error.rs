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

use super::{ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use aldrin_proto::Message;
use futures_channel::mpsc::SendError;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ConnectError {
    UnexpectedEof,
    VersionMismatch(u32),
    UnexpectedMessageReceived(Message),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectError::UnexpectedEof => f.write_str("unexpected end of file"),
            ConnectError::VersionMismatch(v) => {
                f.write_fmt(format_args!("broker version {} mismatch", v))
            }
            ConnectError::UnexpectedMessageReceived(_) => {
                f.write_str("unexpected message received")
            }
        }
    }
}

impl StdError for ConnectError {}

#[derive(Debug, Clone)]
pub enum RunError {
    InternalError,
    UnexpectedMessageReceived(Message),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunError::InternalError => f.write_str("internal error"),
            RunError::UnexpectedMessageReceived(_) => f.write_str("unexpected message received"),
        }
    }
}

impl StdError for RunError {}

#[derive(Debug, Clone)]
pub enum Error {
    InternalError,
    ClientShutdown,
    DuplicateObject(ObjectUuid),
    InvalidObject(ObjectId),
    DuplicateService(ObjectId, ServiceUuid),
    InvalidService(ServiceId),
    InvalidFunction(ServiceId, u32),
    InvalidArgs(ServiceId, u32),
    FunctionCallAborted,
    MissingRequiredField,
}

impl From<SendError> for Error {
    fn from(e: SendError) -> Self {
        if e.is_disconnected() {
            Error::ClientShutdown
        } else {
            Error::InternalError
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InternalError => f.write_str("internal error"),
            Error::ClientShutdown => f.write_str("client shutdown"),
            Error::DuplicateObject(uuid) => f.write_fmt(format_args!("duplicate object {}", uuid)),
            Error::InvalidObject(id) => f.write_fmt(format_args!("invalid object {}", id.uuid)),
            Error::DuplicateService(obj_id, uuid) => f.write_fmt(format_args!(
                "duplicate service {} for object {}",
                uuid, obj_id.uuid,
            )),
            Error::InvalidService(id) => f.write_fmt(format_args!(
                "invalid service {} for object {}",
                id.uuid, id.object_id.uuid,
            )),
            Error::InvalidFunction(id, func) => f.write_fmt(format_args!(
                "invalid function {} of service {} and object {}",
                func, id.uuid, id.object_id.uuid,
            )),
            Error::InvalidArgs(id, func) => f.write_fmt(format_args!(
                "invalid args for function {} of service {} and object {}",
                func, id.uuid, id.object_id.uuid,
            )),
            Error::FunctionCallAborted => f.write_str("function call aborted"),
            Error::MissingRequiredField => f.write_str("required field missing"),
        }
    }
}

impl StdError for Error {}
