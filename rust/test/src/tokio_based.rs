//! Tokio-based utilities for Aldrin server and client tests
//!
//! The types in this module are conceptually identical to the ones in the top-level crate, but are
//! more convenient if you use Tokio, because they all automatically spawn the required tasks.
//!
//! # Examples
//!
//! This is again the calculator example. See the [crate-level documentation](crate) for the Aldrin
//! schema.
//!
//! ```
//! use aldrin_client::{Handle, ObjectUuid};
//! use aldrin_test::tokio_based::TestBroker;
//! use anyhow::Result;
//! use futures::StreamExt;
//!
//! aldrin_codegen_macros::generate!("examples/calculator.aldrin");
//!
//! // Assume this is the #[test] (or #[tokio::test]) function.
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     use calculator::{CALCULATOR_UUID, CalculatorAddArgs, CalculatorProxy};
//!
//!     // Create a broker and client:
//!     let mut broker = TestBroker::new();
//!     let mut client = broker.add_client().await;
//!
//!     // Run our calculator server:
//!     tokio::spawn(run_calculator(client.clone()));
//!
//!     // Create a proxy to the calculator:
//!     let service_id = client.wait_for_service(CALCULATOR_UUID, None).await?;
//!     let calc = CalculatorProxy::bind(client.clone(), service_id).await?;
//!
//!     // Perform a few tests:
//!     assert_eq!(calc.add(CalculatorAddArgs { lhs: 1, rhs: 2 })?.await?.unwrap(), 3);
//!     assert_eq!(
//!         calc.add(CalculatorAddArgs { lhs: i32::MIN, rhs: i32::MAX })?.await?.unwrap(),
//!         -1,
//!     );
//!     assert!(calc.add(CalculatorAddArgs { lhs: i32::MAX, rhs: 1 })?.await?.is_err());
//!     assert!(calc.add(CalculatorAddArgs { lhs: i32::MIN, rhs: -1 })?.await?.is_err());
//!
//!     // Shut everything down:
//!     client.join().await;
//!     broker.join().await;
//!     Ok(())
//! }
//!
//! /// Creates and runs a calculator.
//! async fn run_calculator(handle: Handle) -> Result<()> {
//!     use calculator::{Calculator, CalculatorAddError, CalculatorFunction};
//!
//!     let object = handle.create_object(ObjectUuid::new_v4()).await?;
//!     let mut calc = calculator::Calculator::create(&object).await?;
//!
//!     // There is just a single function, thus not need to `match`.
//!     while let Some(CalculatorFunction::Add(args, reply)) = calc.next().await.transpose()? {
//!         match args.lhs.checked_add(args.rhs) {
//!             Some(res) => reply.ok(res)?,
//!             None => reply.err(CalculatorAddError::Overflow)?,
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

#[cfg(test)]
mod test;

use aldrin_broker::{BrokerHandle, ConnectionError, ConnectionHandle};
use aldrin_channel::Disconnected;
use aldrin_client::{Handle, RunError};
use std::ops::{Deref, DerefMut};
use tokio::task::JoinHandle;

/// Tokio-based broker for use in tests.
///
/// This type is a simple wrapper around [`aldrin_broker::Broker`] and
/// [`aldrin_broker::BrokerHandle`]. All method of [`BrokerHandle`] can be called on this type as
/// well due to its [`Deref`] implementation.
///
/// See the [`tokio_based` module documentation](self) for usage examples.
#[derive(Debug)]
pub struct TestBroker {
    /// Handle to the broker.
    pub handle: BrokerHandle,

    join: Option<JoinHandle<()>>,
}

impl TestBroker {
    /// Creates a new broker.
    pub fn new() -> Self {
        let mut broker = crate::TestBroker::new();
        let join = tokio::spawn(broker.take_broker().run());
        TestBroker {
            handle: broker.handle,
            join: Some(join),
        }
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
        self.handle.shutdown().await;
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
        self.handle.shutdown_idle().await;
        self.join.take().expect("already joined").await.unwrap();
    }

    /// Creates a new `ClientBuilder`.
    ///
    /// Use of this function is recommended only if non-default settings are required for the
    /// [`Client`](aldrin_client::Client) or its [`Connection`](aldrin_broker::Connection). A
    /// default [`Client`](aldrin_client::Client) can be added directly with
    /// [`add_client`](TestBroker::add_client).
    pub fn client_builder(&self) -> ClientBuilder {
        ClientBuilder::new(self.handle.clone())
    }

    /// Creates a default `Client`.
    ///
    /// If you need more control over the [`Client`](aldrin_client::Client) and
    /// [`Connection`](aldrin_broker::Connection) settings, then use
    /// [`client_builder`](TestBroker::client_builder) instead.
    pub async fn add_client(&self) -> TestClient {
        self.client_builder().build().await
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

/// Tokio-based builder struct for a new `Client`.
///
/// A [`ClientBuilder`] allows for more control over how [`Client`](aldrin_client::Client) and
/// [`Connection`](aldrin_broker::Connection) are setup, specifically what kind of channel is used
/// as the transport. If you do not required any special settings, it is recommended to use
/// [`TestBroker::add_client`] instead.
#[derive(Debug, Clone)]
pub struct ClientBuilder(crate::ClientBuilder);

impl ClientBuilder {
    /// Creates a new `ClientBuilder`.
    ///
    /// The default [`ClientBuilder`] is configured to use an unbounded channel between
    /// [`Broker`](aldrin_broker::Broker) and [`Client`](aldrin_client::Client).
    pub fn new(broker: BrokerHandle) -> Self {
        ClientBuilder(crate::ClientBuilder::new(broker))
    }

    /// Creates a new `TestClient` with the current settings.
    pub async fn build(self) -> TestClient {
        let mut client = self.0.build().await;
        TestClient {
            client: Some(tokio::spawn(client.take_client().run())),
            conn: Some(tokio::spawn(client.take_connection().run())),
            handle: client.handle,
            connection_handle: client.connection_handle,
        }
    }

    /// Uses an unbounded channel as the transport between `Broker` and `Client`.
    ///
    /// This is the default after creating a new [`ClientBuilder`].
    #[must_use = "this method follows the builder pattern and returns a new `ClientBuilder`"]
    pub fn unbounded_channel(mut self) -> Self {
        self.0 = self.0.unbounded_channel();
        self
    }

    /// Uses a bounded channel as the transport between `Broker` and `Client`.
    ///
    /// See [`aldrin_channel::bounded`] for more information on the `fifo_size` parameter.
    #[must_use = "this method follows the builder pattern and returns a new `ClientBuilder`"]
    pub fn bounded_channel(mut self, fifo_size: usize) -> Self {
        self.0 = self.0.bounded_channel(fifo_size);
        self
    }
}

/// Tokio-based client for use in tests.
///
/// [`TestClient`s](TestClient) with default settings can be created using
/// [`TestBroker::add_client`], or alternatively with a [`ClientBuilder`] if more control over the
/// settings is required.
///
/// [`TestClient`] dereferences to [`aldrin_client::Handle`] and thus all methods on [`Handle`] can
/// be called on [`TestClient`] as well.
#[derive(Debug)]
pub struct TestClient {
    /// `Handle` to the `Client`.
    pub handle: Handle,

    /// `ConnectionHandle` to the `Connection`.
    pub connection_handle: ConnectionHandle,

    client: Option<JoinHandle<Result<(), RunError<Disconnected>>>>,
    conn: Option<JoinHandle<Result<(), ConnectionError<Disconnected>>>>,
}

impl TestClient {
    /// Creates a new `ClientBuilder`.
    pub fn builder(broker: BrokerHandle) -> ClientBuilder {
        ClientBuilder::new(broker)
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
        self.handle.shutdown();
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
        &self.handle
    }
}
