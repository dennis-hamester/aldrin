//! Low-level types
//!
//! The types in this module are primarily intended for use by the code generator.

mod call;
mod channel;
mod event;
mod promise;
mod proxy;
mod reply;
mod service;
mod service_info;
#[cfg(test)]
mod test;

pub(crate) use proxy::ProxyId;
pub(crate) use service::RawCall;

pub use call::Call;
pub use channel::{
    ChannelBuilder, PendingReceiver, PendingSender, Receiver, Sender, UnboundReceiver,
    UnboundSender, UnclaimedReceiver, UnclaimedSender,
};
pub use event::Event;
pub use promise::Promise;
pub use proxy::Proxy;
pub use reply::Reply;
pub use service::Service;
pub use service_info::ServiceInfo;
