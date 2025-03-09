//! Error types.

use crate::{UnknownCall, UnknownEvent};
use aldrin_core::message::Message;
use aldrin_core::{DeserializeError, SerializeError, SerializedValue};
use thiserror::Error;

/// Error when connecting to a broker.
#[derive(Error, Debug, Clone)]
pub enum ConnectError<T> {
    /// The protocol version of the broker is incompatible.
    #[error("incompatible protocol version")]
    IncompatibleVersion,

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
    Rejected(Option<SerializedValue>),

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

    /// A value failed to serialize.
    #[error(transparent)]
    Serialize(SerializeError),

    /// A value failed to deserialize.
    #[error(transparent)]
    Deserialize(DeserializeError),
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

    /// An invalid event was received.
    ///
    /// This can indicate a schema mismatch.
    ///
    /// This error is never automatically generated. Functions that receive events (such as
    /// [`Proxy::next_event`](crate::low_level::Proxy::next_event)) will normally skip over any
    /// errors. If however event fallbacks are used, then [`UnknownEvent`] can be manually converted
    /// to this variant.
    #[error(transparent)]
    InvalidEvent(#[from] InvalidEvent),

    /// Invalid arguments were supplied to a function or event.
    ///
    /// This can indicate a schema mismatch.
    #[error(transparent)]
    InvalidArguments(#[from] InvalidArguments),

    /// A call was aborted.
    #[error("call aborted")]
    CallAborted,

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

    /// The negotiated protocol version is too low.
    #[error("not supported")]
    NotSupported,
}

impl Error {
    /// Creates a new `InvalidFunction` error.
    pub fn invalid_function(id: u32) -> Self {
        Self::InvalidFunction(InvalidFunction::new(id))
    }

    /// Creates a new `InvalidArguments` error.
    pub fn invalid_arguments(id: u32, source: Option<DeserializeError>) -> Self {
        Self::InvalidArguments(InvalidArguments::new(id, source))
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

impl From<UnknownCall> for Error {
    fn from(call: UnknownCall) -> Self {
        InvalidFunction::from(call).into()
    }
}

impl From<&UnknownCall> for Error {
    fn from(call: &UnknownCall) -> Self {
        InvalidFunction::from(call).into()
    }
}

impl From<UnknownEvent> for Error {
    fn from(event: UnknownEvent) -> Self {
        InvalidEvent::from(event).into()
    }
}

impl From<&UnknownEvent> for Error {
    fn from(event: &UnknownEvent) -> Self {
        InvalidEvent::from(event).into()
    }
}

/// An invalid function was called.
///
/// This can indicate a schema mismatch.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid function {id} called")]
pub struct InvalidFunction {
    id: u32,
}

impl InvalidFunction {
    /// Creates a new `InvalidFunction` error.
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    /// Returns the id of the invalid function.
    pub fn id(self) -> u32 {
        self.id
    }
}

impl From<UnknownCall> for InvalidFunction {
    fn from(call: UnknownCall) -> Self {
        Self::new(call.id())
    }
}

impl From<&UnknownCall> for InvalidFunction {
    fn from(call: &UnknownCall) -> Self {
        Self::new(call.id())
    }
}

/// An invalid event was received.
///
/// This can indicate a schema mismatch.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid event {id} received")]
pub struct InvalidEvent {
    id: u32,
}

impl InvalidEvent {
    /// Creates a new `InvalidEvent` error.
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    /// Returns the id of the invalid event.
    pub fn id(self) -> u32 {
        self.id
    }
}

impl From<UnknownEvent> for InvalidEvent {
    fn from(event: UnknownEvent) -> Self {
        Self::new(event.id())
    }
}

impl From<&UnknownEvent> for InvalidEvent {
    fn from(event: &UnknownEvent) -> Self {
        Self::new(event.id())
    }
}

/// Invalid arguments were supplied to a function or event.
///
/// This can indicate a schema mismatch.
#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("invalid arguments for function or event {id}")]
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
