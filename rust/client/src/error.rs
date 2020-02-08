use super::{ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use aldrin_proto::Message;
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
    UnexpectedFunctionReply,
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
            Error::UnexpectedFunctionReply => f.write_str("unexpected function reply"),
        }
    }
}

impl StdError for Error {}
