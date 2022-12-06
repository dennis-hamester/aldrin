//! Aldrin broker library
//!
//! This library implements the messaging broker according to the Aldrin specification. An Aldrin
//! broker sits at the center of every Aldrin bus and manages the state of all objects and services.
//!
//! The central [`Broker`] is completely transport-agnostic, in the sense that it doesn't know or
//! deal with the low-level details of how a client is connected. E.g. whether TCP/IP with JSON
//! serialization is used or some other means. These details are encapsulated in [`Connection`],
//! which in turn communicates with the `Broker` through internal channels.
//!
//! Furthermore, this crate does not depend on any async runtime, such as e.g. Tokio. Neither the
//! `Broker` nor `Connection` need to spawn additional tasks, nor perform any I/O on their
//! own. Users of this crate have full control over what runtime to use (if any at all) and how to
//! arrange the various parts into tasks.
//!
//! # Examples
//!
//! A typical use-case is to have a stand-alone broker application, which simply listens for new
//! connections in an infinite loop. In this example, we'll be using Tokio and TCP/IP connections.
//!
//! ```
//! use aldrin_broker::Broker;
//! use aldrin_proto::tokio::TokioTransport;
//! use anyhow::Result;
//! use std::net::Ipv4Addr;
//! use tokio::net::TcpListener;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create and spawn the broker:
//!     let broker = Broker::new();
//!     let handle = broker.handle().clone();
//!     tokio::spawn(broker.run());
//!
//!     // Create a TcpListener to listen for new connections:
//!     let mut listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), 24387)).await?;
//!
//!     loop {
//!         # return Ok(());
//!         let (socket, _) = listener.accept().await?;
//!
//!         // Create an AsyncTransport out of the socket:
//!         let transport = TokioTransport::new(socket);
//!
//!         // Add the connection and run it:
//!         let connection = handle.connect(transport).await?;
//!         tokio::spawn(connection.run());
//!     }
//! }
//! ```

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

mod broker;
mod conn;
mod conn_id;
mod serial_map;

#[cfg(feature = "statistics")]
pub use broker::BrokerStatistics;
pub use broker::{Broker, BrokerHandle, BrokerShutdown, PendingConnection};
pub use conn::{Connection, ConnectionError, ConnectionHandle, EstablishError};
