//! Utilities for Aldrin server and client tests
//!
//! This crate provides a simple way to quickly setup a complete Aldrin bus with a broker and
//! multiple clients and is intended to be used in unit tests.
//!
//! If you are using Tokio, it is strongly recommended to enable this crate's `tokio` feature and
//! use the types in the `tokio` module instead of the crate-level types.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

#[cfg(test)]
mod test;

#[cfg(feature = "tokio")]
pub mod tokio;

use aldrin::{Client, Handle};
use aldrin_broker::{Broker, BrokerHandle, Connection, ConnectionHandle};
use aldrin_core::channel::{self, Disconnected};
use aldrin_core::transport::{AsyncTransportExt, BoxedTransport};
use futures_util::future;
use std::ops::{Deref, DerefMut};

// For tests directly in aldrin_broker and aldrin.
#[doc(hidden)]
pub use {aldrin, aldrin_broker};

/// Broker for use in tests.
///
/// This type is a simple wrapper around [`aldrin_broker::Broker`] and
/// [`aldrin_broker::BrokerHandle`]. All method of [`BrokerHandle`] can be called on this type as
/// well due to its [`Deref`] implementation.
///
/// The underlying [`Broker`] is not automatically started. Users must
/// [take the `broker`](TestBroker::take_broker) and [run](Broker::run) it manually.
///
/// See the [crate-level documentation](crate) for usage examples.
#[derive(Debug)]
pub struct TestBroker {
    handle: BrokerHandle,
    broker: Option<Broker>,
}

impl TestBroker {
    /// Creates a new broker.
    pub fn new() -> Self {
        let broker = Broker::new();

        TestBroker {
            handle: broker.handle().clone(),
            broker: Some(broker),
        }
    }

    /// Returns a handle to the broker.
    pub fn handle(&self) -> &BrokerHandle {
        &self.handle
    }

    /// Takes the broker out of this struct.
    ///
    /// After creating a [`TestBroker`], the actual [`Broker`] must be taken out and
    /// [run](Broker::run).
    ///
    /// # Panics
    ///
    /// This function panics when the [`Broker`] has already been taken out. It must be called only
    /// once.
    pub fn take_broker(&mut self) -> Broker {
        self.broker.take().expect("broker already taken")
    }

    /// Add a new client to the broker.
    pub async fn add_client(&mut self) -> TestClient {
        let (t1, t2) = channel::unbounded();

        let client = Client::connect(t1.boxed());
        let conn = self.handle.connect(t2.boxed());

        let (client, conn) = future::join(client, conn).await;
        let client = client.expect("client failed to connect");
        let handle = client.handle().clone();
        let conn = conn.expect("connection failed to establish");
        let connection_handle = conn.handle().clone();

        TestClient::new(handle, connection_handle, client, conn)
    }
}

impl Default for TestBroker {
    fn default() -> Self {
        TestBroker::new()
    }
}

impl Deref for TestBroker {
    type Target = BrokerHandle;

    fn deref(&self) -> &BrokerHandle {
        &self.handle
    }
}

impl DerefMut for TestBroker {
    fn deref_mut(&mut self) -> &mut BrokerHandle {
        &mut self.handle
    }
}

/// Client for use in tests.
///
/// After creating a [`TestClient`], it is fully connected to the [`TestBroker`], but neither the
/// underlying [`Client`] nor the [`Connection`] are running. This must be done manually by taking
/// both parts out and calling their respective `run` methods (see
/// [`take_client`](Self::take_client) and [`take_connection`](Self::take_connection).
///
/// [`TestClient`] dereferences to [`aldrin::Handle`] and thus all methods on [`Handle`] can
/// be called on [`TestClient`] as well.
#[derive(Debug)]
pub struct TestClient {
    handle: Handle,
    connection_handle: ConnectionHandle,
    client: Option<Client<BoxedTransport<'static, Disconnected>>>,
    conn: Option<Connection<BoxedTransport<'static, Disconnected>>>,
}

impl TestClient {
    fn new(
        handle: Handle,
        connection_handle: ConnectionHandle,
        client: Client<BoxedTransport<'static, Disconnected>>,
        conn: Connection<BoxedTransport<'static, Disconnected>>,
    ) -> Self {
        Self {
            handle,
            connection_handle,
            client: Some(client),
            conn: Some(conn),
        }
    }

    /// Returns a handle to the client.
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Returns a handle to the connection.
    pub fn connection(&self) -> &ConnectionHandle {
        &self.connection_handle
    }

    /// Takes the `Client` out of this struct.
    ///
    /// After creating a [`TestClient`], the actual [`Client`] must be taken out and
    /// [run](Client::run).
    ///
    /// # Panics
    ///
    /// This function panics when the [`Client`] has already been taken out. It must be called only
    /// once.
    pub fn take_client(&mut self) -> Client<BoxedTransport<'static, Disconnected>> {
        self.client.take().expect("client already taken")
    }

    /// Takes the `Connection` out of this struct.
    ///
    /// After creating a [`TestClient`], the actual [`Connection`] must be taken out and
    /// [run](Connection::run).
    ///
    /// # Panics
    ///
    /// This function panics when the [`Connection`] has already been taken out. It must be called
    /// only once.
    pub fn take_connection(&mut self) -> Connection<BoxedTransport<'static, Disconnected>> {
        self.conn.take().expect("connection already taken")
    }
}

impl Deref for TestClient {
    type Target = Handle;

    fn deref(&self) -> &Handle {
        &self.handle
    }
}
