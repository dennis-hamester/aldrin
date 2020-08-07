use aldrin_proto::AsyncTransport;
use aldrin_util::channel::{Channel, Disconnected, Unbounded};
use std::fmt::Debug;

/// Asynchronous transport used for connections between test brokers and clients.
///
/// This trait is sealed and cannot be implemented for types outside of this crate. The transports
/// used in this crate are always based on the channels available in [`aldrin_util::channel`]. The
/// actual transport type used in this crate is `Box<dyn TestTransport>`.
pub trait TestTransport:
    Sealed + AsyncTransport<Error = Disconnected> + Debug + Unpin + Send
{
}

impl TestTransport for Channel {}
impl TestTransport for Unbounded {}

pub trait Sealed {}

impl Sealed for Channel {}
impl Sealed for Unbounded {}
