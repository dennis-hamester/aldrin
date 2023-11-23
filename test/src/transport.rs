use aldrin_proto::channel::{Bounded, Disconnected, Unbounded};
use aldrin_proto::transport::AsyncTransport;
use std::fmt::Debug;

/// Asynchronous transport used for connections between test brokers and clients.
///
/// This trait is sealed and cannot be implemented for types outside of this crate. The transports
/// used in this crate are always based on the channels available in [`aldrin_proto::channel`]. The
/// actual transport type used in this crate is `Box<dyn TestTransport>`.
pub trait TestTransport:
    Sealed + AsyncTransport<Error = Disconnected> + Debug + Unpin + Send + Sync
{
}

impl TestTransport for Bounded {}
impl TestTransport for Unbounded {}

pub trait Sealed {}

impl Sealed for Bounded {}
impl Sealed for Unbounded {}
