//! Tokio-based utilities for Aldrin server and client tests
//!
//! The types in this module are conceptually identical to the ones in the top-level crate, but are
//! more convenient if you use Tokio, because they all automatically spawn the required tasks.

#[cfg(test)]
mod test;

use aldrin::error::RunError;
use aldrin::Handle;
use aldrin_broker::{BrokerHandle, ConnectionError, ConnectionHandle};
use aldrin_core::channel::Disconnected;
use std::ops::{Deref, DerefMut};
use tokio::task::JoinHandle;

/// Tokio-based broker for use in tests.
///
/// This type is a simple wrapper around [`aldrin_broker::Broker`] and
/// [`aldrin_broker::BrokerHandle`]. All method of [`BrokerHandle`] can be called on this type as
/// well due to its [`Deref`] implementation.
///
/// See the [`tokio` module documentation](self) for usage examples.
#[derive(Debug)]
pub struct TestBroker {
    inner: crate::TestBroker,
    join: Option<JoinHandle<()>>,
}

impl TestBroker {
    /// Creates a new broker.
    pub fn new() -> Self {
        let mut inner = crate::TestBroker::new();

        TestBroker {
            join: Some(tokio::spawn(inner.take_broker().run())),
            inner,
        }
    }

    /// Returns a handle to the broker.
    pub fn handle(&self) -> &BrokerHandle {
        &self.inner
    }

    /// Shuts down the broker and joins its task.
    ///
    /// This function cannot be canceled in a meaningful way after it has been polled once, because
    /// it would panic if called (and polled) again. Ensure that you call it only once and then poll
    /// it to completion.
    ///
    /// # Panics
    ///
    /// This function will panic if the [`Broker`](aldrin_broker::Broker) task has already been
    /// joined or attempted to join (see notes above as well).
    pub async fn join(&mut self) {
        self.inner.shutdown().await;
        self.join.take().expect("already joined").await.unwrap();
    }

    /// Shuts down the broker when idle and joins its task.
    ///
    /// This function cannot be canceled in a meaningful way after it has been polled once, because
    /// it would panic if called (and polled) again. Ensure that you call it only once and then poll
    /// it to completion.
    ///
    /// # Panics
    ///
    /// This function will panic if the [`Broker`](aldrin_broker::Broker) task has already been
    /// joined or attempted to join (see notes above as well).
    pub async fn join_idle(&mut self) {
        self.inner.shutdown_idle().await;
        self.join.take().expect("already joined").await.unwrap();
    }

    /// Creates a new `Client`.
    pub async fn add_client(&mut self) -> TestClient {
        let inner = self.inner.add_client().await;
        TestClient::new(inner)
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
        &self.inner
    }
}

impl DerefMut for TestBroker {
    fn deref_mut(&mut self) -> &mut BrokerHandle {
        &mut self.inner
    }
}

/// Tokio-based client for use in tests.
///
/// [`TestClient`] dereferences to [`aldrin::Handle`] and thus all methods on [`Handle`] can
/// be called on [`TestClient`] as well.
#[derive(Debug)]
pub struct TestClient {
    inner: crate::TestClient,
    client: Option<JoinHandle<Result<(), RunError<Disconnected>>>>,
    conn: Option<JoinHandle<Result<(), ConnectionError<Disconnected>>>>,
}

impl TestClient {
    fn new(mut inner: crate::TestClient) -> Self {
        Self {
            client: Some(tokio::spawn(inner.take_client().run())),
            conn: Some(tokio::spawn(inner.take_connection().run())),
            inner,
        }
    }

    /// Returns a handle to the client.
    pub fn handle(&self) -> &Handle {
        &self.inner
    }

    /// Returns a handle to the connection.
    pub fn connection(&self) -> &ConnectionHandle {
        self.inner.connection()
    }

    /// Shuts down the client and joins the client and connection tasks.
    ///
    /// This function cannot be canceled in a meaningful way after it has been polled once, because
    /// it would panic if called (and polled) again. Ensure you call it only once and then poll it
    /// to completion.
    ///
    /// # Panics
    ///
    /// This function will panic if the tasks have already been joined or attempted to join (see
    /// notes above as well).
    pub async fn join(&mut self) {
        self.inner.shutdown();
        self.join_client().await;
        self.join_connection().await;
    }

    async fn join_client(&mut self) {
        self.client
            .take()
            .expect("client already joined")
            .await
            .unwrap()
            .unwrap();
    }

    async fn join_connection(&mut self) {
        self.conn
            .take()
            .expect("connection already joined")
            .await
            .unwrap()
            .unwrap();
    }
}

impl Deref for TestClient {
    type Target = Handle;

    fn deref(&self) -> &Handle {
        &self.inner
    }
}
