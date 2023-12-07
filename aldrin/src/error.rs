//! Error types.

use crate::core::message::Message;
use crate::core::{DeserializeError, SerializeError, SerializedValue};
use thiserror::Error;

/// Error when connecting to a broker.
#[derive(Error, Debug, Clone)]
pub enum ConnectError<T> {
    /// The protocol version of the broker is incompatible.
    ///
    /// The contained version is that of the broker. The version of the client is
    /// [`crate::core::VERSION`].
    #[error("incompatible protocol version {}", .0)]
    IncompatibleVersion(u32),

    /// An unexpected message was received.
    ///
    /// This is usually an indication of a bug in the broker.
    #[error("unexpected message received")]
    UnexpectedMessageReceived(Message),

    /// The transport returned an error.
    ///
    /// This error indicates some issue with the lower-level transport mechanism, e.g. an I/O error.
    #[error(transparent)]
    Transport(T),

    /// The broker rejected the connection.
    #[error("connection rejected")]
    Rejected(SerializedValue),

    /// A value failed to serialize.
    #[error(transparent)]
    Serialize(#[from] SerializeError),

    /// A value failed to deserialize.
    #[error(transparent)]
    Deserialize(#[from] DeserializeError),
}

/// Error while running a client.
#[derive(Error, Debug, Clone)]
pub enum RunError<T> {
    /// An unexpected message was received.
    #[error("unexpected message received")]
    UnexpectedMessageReceived(Message),

    /// The transport returned an error.
    ///
    /// This error indicates some issue with the lower-level transport mechanism, e.g. an I/O error.
    #[error(transparent)]
    Transport(#[from] T),
}

/// Standard error type used for most functions.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    /// The client has shut down.
    #[error("client shut down")]
    Shutdown,

    /// An object could not be created because its UUID exists already.
    #[error("duplicate object")]
    DuplicateObject,

    /// An invalid object id was used.
    ///
    /// This typically indicates that the object was destroyed.
    #[error("invalid object")]
    InvalidObject,

    /// An service could not be created because its UUID exists already on the object.
    #[error("duplicate service")]
    DuplicateService,

    /// An invalid service id was used.
    ///
    /// This typically indicates that the service or owning object was destroyed.
    #[error("invalid service")]
    InvalidService,

    /// An invalid function was called.
    ///
    /// This can indicate a schema mismatch.
    #[error(transparent)]
    InvalidFunction(#[from] InvalidFunction),

    /// Invalid arguments were supplied to a function or event.
    ///
    /// This can indicate a schema mismatch.
    #[error(transparent)]
    InvalidArguments(#[from] InvalidArguments),

    /// A call was aborted.
    #[error("call aborted")]
    CallAborted,

    /// A field that is required for some type is missing.
    #[error(transparent)]
    RequiredFieldMissing(#[from] RequiredFieldMissing),

    /// An invalid reply was received for a call.
    ///
    /// This can indicate a schema mismatch.
    #[error(transparent)]
    InvalidReply(#[from] InvalidReply),

    /// An invalid channel was used.
    #[error("invalid channel")]
    InvalidChannel,

    /// An invalid item was received on a channel.
    ///
    /// This can indicate a schema mismatch.
    #[error(transparent)]
    InvalidItem(#[from] InvalidItem),

    /// An invalid bus was used.
    #[error("invalid bus listener")]
    InvalidBusListener,

    /// A bus listener was started, that is already started.
    #[error("bus listener already started")]
    BusListenerAlreadyStarted,

    /// A bus listener was stopped, that is already stopped.
    #[error("bus listener not started")]
    BusListenerNotStarted,

    /// An invalid lifetime was used.
    #[error("invalid lifetime")]
    InvalidLifetime,

    /// A value failed to serialized.
    #[error(transparent)]
    Serialize(#[from] SerializeError),
}

impl Error {
    /// Creates a new `InvalidFunction` error.
    pub fn invalid_function(function: u32) -> Self {
        Self::InvalidFunction(InvalidFunction::new(function))
    }

    /// Creates a new `InvalidArguments` error.
    pub fn invalid_arguments(id: u32, source: Option<DeserializeError>) -> Self {
        Self::InvalidArguments(InvalidArguments::new(id, source))
    }

    /// Creates a new `RequiredFieldMissing` error.
    pub fn required_field_missing(field: u32) -> Self {
        Self::RequiredFieldMissing(RequiredFieldMissing::new(field))
    }

    /// Creates a new `InvalidReply` error.
    pub fn invalid_reply(source: DeserializeError) -> Self {
        Self::InvalidReply(InvalidReply::new(source))
    }

    /// Creates a new `InvalidItem` error.
    pub fn invalid_item(source: DeserializeError) -> Self {
        Self::InvalidItem(InvalidItem::new(source))
    }
}

/// An invalid function was called.
///
/// This can indicate a schema mismatch.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid function {} called", .function)]
pub struct InvalidFunction {
    function: u32,
}

impl InvalidFunction {
    /// Creates a new `InvalidFunction` error.
    pub fn new(function: u32) -> Self {
        Self { function }
    }

    /// Returns the id of the invalid function.
    pub fn function(self) -> u32 {
        self.function
    }
}

/// Invalid arguments were supplied to a function or event.
///
/// This can indicate a schema mismatch.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid arguments for function or event {}", .id)]
pub struct InvalidArguments {
    id: u32,
    source: Option<DeserializeError>,
}

impl InvalidArguments {
    /// Creates a new `InvalidArguments` error.
    pub fn new(id: u32, source: Option<DeserializeError>) -> Self {
        Self { id, source }
    }

    /// Returns the id of the function or event.
    pub fn id(self) -> u32 {
        self.id
    }
}

/// A field that is required for some type is missing.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("required field {} missing", .field)]
pub struct RequiredFieldMissing {
    field: u32,
}

impl RequiredFieldMissing {
    /// Creates a new `RequiredFieldMissing` error.
    pub fn new(field: u32) -> Self {
        Self { field }
    }

    /// Returns the id of the field that is missing.
    pub fn field(self) -> u32 {
        self.field
    }
}

/// An invalid reply was received for a call.
///
/// This can indicate a schema mismatch.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid reply")]
pub struct InvalidReply {
    #[from]
    source: DeserializeError,
}

impl InvalidReply {
    /// Creates a new `InvalidReply` error.
    pub fn new(source: DeserializeError) -> Self {
        Self { source }
    }
}

/// An invalid item was received on a channel.
///
/// This can indicate a schema mismatch.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid item")]
pub struct InvalidItem {
    #[from]
    source: DeserializeError,
}

impl InvalidItem {
    /// Creates a new `InvalidItem` error.
    pub fn new(source: DeserializeError) -> Self {
        Self { source }
    }
}
