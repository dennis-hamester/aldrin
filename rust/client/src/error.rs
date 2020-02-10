use super::{ObjectId, ObjectUuid, ServiceId, ServiceUuid};
use aldrin_proto::Message;
use std::error::Error as StdError;
use std::fmt;

/// Error while connecting to a broker.
#[derive(Debug, Clone)]
pub enum ConnectError<T> {
    /// The connection was closed unexpectedly.
    UnexpectedEof,

    /// The Aldrin protocol version differs between client and broker.
    ///
    /// The contained version is that of the broker. The version of the client is
    /// [`aldrin_proto::VERSION`].
    VersionMismatch(u32),

    /// An unexpected message was received.
    UnexpectedMessageReceived(Message),

    /// The transport returned an error.
    ///
    /// This should usually be the error type defined by the [`Transport`](aldrin_proto::Transport).
    Transport(T),
}

impl<T> From<T> for ConnectError<T> {
    fn from(e: T) -> Self {
        ConnectError::Transport(e)
    }
}

impl<T> fmt::Display for ConnectError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectError::UnexpectedEof => f.write_str("unexpected end of file"),
            ConnectError::VersionMismatch(v) => {
                f.write_fmt(format_args!("broker version {} mismatch", v))
            }
            ConnectError::UnexpectedMessageReceived(_) => {
                f.write_str("unexpected message received")
            }
            ConnectError::Transport(e) => e.fmt(f),
        }
    }
}

impl<T> StdError for ConnectError<T> where T: fmt::Debug + fmt::Display {}

/// Error while running a client.
#[derive(Debug, Clone)]
pub enum RunError<T> {
    /// Internal error.
    ///
    /// This indicates an internal error that is otherwise not expected nor handled. This should
    /// never happen and probably indicates a bug if it does. Please open a ticket.
    ///
    /// We might remove this variant in the future.
    InternalError,

    /// An unexpected message was received.
    UnexpectedMessageReceived(Message),

    /// The transport returned an error.
    ///
    /// This should usually be the error type defined by the [`Transport`](aldrin_proto::Transport).
    Transport(T),
}

impl<T> From<T> for RunError<T> {
    fn from(e: T) -> Self {
        RunError::Transport(e)
    }
}

impl<T> fmt::Display for RunError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunError::InternalError => f.write_str("internal error"),
            RunError::UnexpectedMessageReceived(_) => f.write_str("unexpected message received"),
            RunError::Transport(e) => e.fmt(f),
        }
    }
}

impl<T> StdError for RunError<T> where T: fmt::Debug + fmt::Display {}

/// Standard error type used for most functions.
#[derive(Debug, Clone)]
pub enum Error {
    /// Internal error.
    ///
    /// This indicates an internal error that is otherwise not expected nor handled. This should
    /// never happen and probably indicates a bug if it does. Please open a ticket.
    ///
    /// We might remove this variant in the future.
    InternalError,

    /// The client has shut down.
    ///
    /// The client can shut down for various reasons, e.g. by explicitly calling
    /// [shutdown](super::Handle::shutdown), or when the broker has shut down. But also errors will
    /// cause the client to shut down and all subsequent requests will return this variant.
    ClientShutdown,

    /// An object could not be created because its [`ObjectUuid`] exists already.
    DuplicateObject(ObjectUuid),

    /// An invalid object id was used.
    ///
    /// This can happen either when trying to [destroy](super::Object::destroy) an object, which has
    /// already been destroyed, or when trying to [create a service](super::Object::create_service)
    /// on a destroyed object.
    InvalidObject(ObjectId),

    /// An service could not be created because its UUID exists already on the object.
    DuplicateService(ObjectId, ServiceUuid),

    /// An invalid service id was used.
    ///
    /// This typically happens when using a service in any way, which has been destroyed, either
    /// [directly](super::Service::destroy), or indirectly, when the object is destroyed.
    InvalidService(ServiceId),

    /// An invalid service function was called.
    ///
    /// This variant is only ever returned by auto-generated code, which a non-existent function has
    /// been called on a service.
    InvalidFunction(ServiceId, u32),

    /// A service function was called with invalid arguments.
    ///
    /// This variant is only ever returned by auto-generated code.
    InvalidArgs(ServiceId, u32),

    /// A service function has been aborted by the callee.
    FunctionCallAborted,

    /// A struct could not be created because of missing required fields.
    ///
    /// This variant is only ever returned by auto-generated code.
    MissingRequiredField,

    /// An unexpected reply to a service function call was received.
    ///
    /// This variant is only ever returned by auto-generated code. It typically means that there is
    /// some issue or difference with the service schemas.
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
