//! Error types.

use crate::core::message::Message;
use crate::core::{
    DeserializeError, ObjectId, ObjectUuid, SerializeError, SerializedValue, ServiceId, ServiceUuid,
};
use crate::lifetime::LifetimeId;
use std::error::Error as StdError;
use std::fmt;

/// Error when connecting to a broker.
#[derive(Debug, Clone)]
pub enum ConnectError<T> {
    /// The Aldrin protocol version differs between client and broker.
    ///
    /// The contained version is that of the broker. The version of the client is
    /// [`crate::core::VERSION`].
    VersionMismatch(u32),

    /// An unexpected message was received.
    ///
    /// This is usually an indication of a bug in the broker.
    UnexpectedMessageReceived(Message),

    /// The transport returned an error.
    ///
    /// This error indicates some issue with the lower-level transport mechanism, e.g. an I/O error.
    Transport(T),

    /// The broker rejected the connection.
    Rejected(SerializedValue),

    /// A value failed to serialize.
    Serialize(SerializeError),

    /// A value failed to deserialize.
    Deserialize(DeserializeError),
}

impl<T> From<SerializeError> for ConnectError<T> {
    fn from(e: SerializeError) -> Self {
        Self::Serialize(e)
    }
}

impl<T> From<DeserializeError> for ConnectError<T> {
    fn from(e: DeserializeError) -> Self {
        Self::Deserialize(e)
    }
}

impl<T> fmt::Display for ConnectError<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectError::VersionMismatch(v) => {
                f.write_fmt(format_args!("broker version {v} mismatch"))
            }
            ConnectError::UnexpectedMessageReceived(_) => {
                f.write_str("unexpected message received")
            }
            ConnectError::Transport(e) => e.fmt(f),
            ConnectError::Rejected(_) => f.write_str("connection rejected"),
            ConnectError::Serialize(e) => e.fmt(f),
            ConnectError::Deserialize(e) => e.fmt(f),
        }
    }
}

impl<T: fmt::Debug + fmt::Display> StdError for ConnectError<T> {}

/// Error while running a client.
#[derive(Debug, Clone)]
pub enum RunError<T> {
    /// An unexpected message was received.
    UnexpectedMessageReceived(Message),

    /// The transport returned an error.
    ///
    /// This error indicates some issue with the lower-level transport mechanism, e.g. an I/O error.
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
            RunError::UnexpectedMessageReceived(_) => f.write_str("unexpected message received"),
            RunError::Transport(e) => e.fmt(f),
        }
    }
}

impl<T: fmt::Debug + fmt::Display> StdError for RunError<T> {}

/// Standard error type used for most functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The client has shut down.
    ///
    /// The client can shut down for various reasons, e.g. by explicitly calling
    /// [`Handle::shutdown`](crate::Handle::shutdown), or when the broker has shut down. But also
    /// errors will cause the client to shut down and all subsequent requests will return this
    /// variant.
    ClientShutdown,

    /// An object could not be created because its UUID exists already.
    DuplicateObject(ObjectUuid),

    /// An invalid object id was used.
    ///
    /// This can happen either when trying to [destroy](crate::Object::destroy) an
    /// [`Object`](crate::Object), which has already been destroyed, or when trying to
    /// [create a service](crate::Object::create_service) on a destroyed object.
    InvalidObject(ObjectId),

    /// An service could not be created because its UUID exists already on the object.
    DuplicateService(ObjectId, ServiceUuid),

    /// An invalid service id was used.
    ///
    /// This typically happens when using a service, which has already been destroyed, either
    /// [directly](super::Service::destroy), or indirectly, when the object is destroyed.
    InvalidService(ServiceId),

    /// An invalid service function was called.
    ///
    /// This variant is only ever returned by auto-generated code, when a non-existent function has
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

    /// A function call was replied to with an invalid result.
    InvalidFunctionResult(InvalidFunctionResult),

    /// An invalid function call was received.
    InvalidFunctionCall(InvalidFunctionCall),

    /// An event was received with invalid arguments.
    InvalidEventArguments(InvalidEventArguments),

    /// An operation was performed on an invalid channel.
    ///
    /// This error happens in the following cases:
    /// - Any use of a closed channel.
    /// - Trying to claim a channel when the other end has been closed.
    /// - When the other channel end gets closed while waiting for the channel to become
    ///   established.
    InvalidChannel,

    /// An operation was performed on a channel that belongs to a different client.
    ///
    /// This error happens in the following cases:
    /// - Trying to close an unclaimed channel that has been claimed by a different client in the
    ///   meantime.
    /// - Trying to claim a channel which has been claimed by another client.
    ForeignChannel,

    /// An item received on a channel failed to deserialize to the expected type.
    InvalidItemReceived,

    /// A value failed to serialize.
    Serialize(SerializeError),

    /// An invalid bus listener was used.
    ///
    /// This typically means that the bus listener was already destroyed.
    InvalidBusListener,

    /// A bus listener was attempted to be started twice.
    BusListenerAlreadyStarted,

    /// A bus listener was attempted to be stopped while it isn't started.
    BusListenerNotStarted,

    /// An invalid lifetime scope was used.
    ///
    /// This can only happen why trying to [end a lifetime scope](crate::LifetimeScope::end), that
    /// has already ended.
    InvalidLifetime(LifetimeId),
}

impl From<SerializeError> for Error {
    fn from(e: SerializeError) -> Self {
        Self::Serialize(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ClientShutdown => f.write_str("client shutdown"),
            Error::DuplicateObject(uuid) => f.write_fmt(format_args!("duplicate object {uuid}")),
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
            Error::InvalidFunctionResult(e) => e.fmt(f),
            Error::InvalidFunctionCall(e) => e.fmt(f),
            Error::InvalidEventArguments(e) => e.fmt(f),
            Error::InvalidChannel => f.write_str("invalid channel"),
            Error::ForeignChannel => f.write_str("foreign channel"),
            Error::InvalidItemReceived => f.write_str("invalid item received"),
            Error::Serialize(e) => e.fmt(f),
            Error::InvalidBusListener => f.write_str("invalid bus listener"),
            Error::BusListenerAlreadyStarted => f.write_str("bus listener already started"),
            Error::BusListenerNotStarted => f.write_str("bus listener not started"),
            Error::InvalidLifetime(id) => {
                f.write_fmt(format_args!("invalid lifetime {}", id.0.uuid))
            }
        }
    }
}

impl StdError for Error {}

/// A function call was replied to with an invalid result.
///
/// For fallible functions invoked via [`Handle::call_function`](crate::Handle::call_function), this
/// error occurs when the result is not convertible to either `T` or `E` (see
/// [`PendingFunctionResult`](crate::PendingFunctionResult)). For infallible function invoked via
/// [`Handle::call_infallible_function`](crate::Handle::call_infallible_function), this error occurs
/// when the result is not convertible to `T` or when it indicates an error (see
/// [`PendingFunctionValue`](crate::PendingFunctionValue)).
///
/// When using auto-generated code, this is typically an indication of an incompatible schema
/// mismatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidFunctionResult {
    /// Id of the service, that was called.
    pub service_id: ServiceId,

    /// Id of the function, that was called.
    pub function: u32,
}

impl fmt::Display for InvalidFunctionResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "function {} of service {} returned an invalid result",
            self.function, self.service_id.uuid,
        ))
    }
}

impl From<InvalidFunctionResult> for Error {
    fn from(e: InvalidFunctionResult) -> Self {
        Error::InvalidFunctionResult(e)
    }
}

impl StdError for InvalidFunctionResult {}

/// An invalid function call was received.
///
/// This error is typically an indication of an incompatible schema mismatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidFunctionCall {
    /// Id of the service, that was called.
    pub service_id: ServiceId,

    /// Id of the function, that was called.
    pub function: u32,
}

impl fmt::Display for InvalidFunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "service {} called with invalid function {} or arguments",
            self.service_id.uuid, self.function,
        ))
    }
}

impl From<InvalidFunctionCall> for Error {
    fn from(e: InvalidFunctionCall) -> Self {
        Error::InvalidFunctionCall(e)
    }
}

impl StdError for InvalidFunctionCall {}

/// An event was received with invalid arguments.
///
/// This error is typically an indication of an incompatible schema mismatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidEventArguments {
    /// Id of the service, that emitted the event.
    pub service_id: ServiceId,

    /// Id of the event.
    pub event: u32,
}

impl fmt::Display for InvalidEventArguments {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "service {} emitted event {} with invalid arguments",
            self.service_id.uuid, self.event,
        ))
    }
}

impl From<InvalidEventArguments> for Error {
    fn from(e: InvalidEventArguments) -> Self {
        Error::InvalidEventArguments(e)
    }
}

impl StdError for InvalidEventArguments {}
