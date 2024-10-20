//! Aldrin client library
//!
//! This library implements the client side of the Aldrin specification and can be used to connect
//! to Aldrin brokers. It is fully asynchronous (async/await) and doesn't depend on any specific
//! async runtime.
//!
//! Parts of this library should be considered low-level building blocks for the code that can be
//! auto-generated from Aldrin schemas. As an example, performing a function call on a service
//! requires knowing the function's id and uses a polymorphic type to encode the function's
//! arguments. It is generally recommended to rely on the more ergonomic auto-generated code
//! instead.
//!
//! The first entry point is the [`Client`] and it's [`connect`](Client::connect)
//! method. [`Client`]s are parameterized over an
//! [`AsyncTransport`](crate::core::transport::AsyncTransport), which abstracts the low-level
//! details of a transport, like e.g. TCP/IP.
//!
//! After establishing a connection, the resulting [`Client`] must be continuously polled (through
//! [`Client::run`]). One way to achieve this is to "spawn" it with an async runtime of your
//! choice. Alternatively, it can also be polled manually.
//!
//! While the [`Client`] is being polled, all interaction with it happens through a [`Handle`],
//! which can be acquired with [`Client::handle`]. The [`Client`] will automatically shut down (as
//! in, the [`Client::run`] future will complete) when the last [`Handle`] has been dropped.
//!
//! # Examples
//!
//! ```
//! use aldrin::Client;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     # let broker = aldrin_test::tokio::TestBroker::new();
//!     # let mut handle = broker.clone();
//!     # let (async_transport, t2) = aldrin::core::channel::unbounded();
//!     # let conn = tokio::spawn(async move { handle.connect(t2).await });
//!     // Create an AsyncTransport for connecting to the broker.
//!     // let async_transport = ...
//!
//!     // Connect to the broker:
//!     let client = Client::connect(async_transport).await?;
//!     # tokio::spawn(conn.await??.run());
//!
//!     // Acquire a handle and spawn the client:
//!     let handle = client.handle().clone();
//!     let join = tokio::spawn(client.run());
//!
//!     // The client is now fully connected and can be interacted with through the handle.
//!
//!     // Shut down client:
//!     handle.shutdown();
//!     join.await??;
//!
//!     Ok(())
//! }
//! ```

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod bus_listener;
mod channel;
mod client;
mod discoverer;
mod function_call_map;
mod handle;
mod lifetime;
mod object;
mod promise;
mod reply;
mod serial_map;
#[cfg(test)]
mod test;

pub mod error;
pub mod low_level;
#[cfg(feature = "codegen")]
#[doc(hidden)]
pub mod private;

pub use aldrin_core as core;
#[cfg(all(feature = "codegen", feature = "introspection"))]
pub use aldrin_macros::IntrospectableFromAldrin as Introspectable;
#[cfg(feature = "codegen")]
pub use aldrin_macros::{
    generate, DeserializeFromAldrin as Deserialize, SerializeFromAldrin as Serialize,
};
pub use bus_listener::BusListener;
pub use channel::{
    PendingReceiver, PendingSender, Receiver, Sender, UnboundReceiver, UnboundSender,
    UnclaimedReceiver, UnclaimedSender,
};
pub use client::Client;
pub use discoverer::{
    Discoverer, DiscovererBuilder, DiscovererEntry, DiscovererEntryIter, DiscovererEvent,
    DiscovererEventKind, DiscovererIter, DiscovererIterEntry,
};
pub use error::Error;
pub use handle::Handle;
pub use lifetime::{Lifetime, LifetimeId, LifetimeScope};
pub use object::Object;
pub use promise::Promise;
pub use reply::Reply;
