//! Utilities for Aldrin server and client tests
//!
//! This crate provides a simple way to quickly setup a complete Aldrin bus with a broker and
//! multiple clients and is intended to be used in unit tests.
//!
//! If you are using Tokio, it is strongly recommended to enable this crate's `tokio` feature and
//! use the type in the `tokio_based` module instead of the crate-level types.
//!
//! # Example
//!
//! In this example, we are writing tests for a calculator service, which can add 2 numbers. Highly
//! sophisticated of course, because it can also handle overflows properly.
//!
//! The calculator schema:
//!
//! ```aldrin
//! service Calculator {
//!     uuid = 50e1ed0e-64a6-45ff-98d3-ce18a04c7292;
//!     version = 1;
//!
//!     fn add @ 1 {
//!         args = struct {
//!             required lhs @ 1 = i32;
//!             required rhs @ 2 = i32;
//!         }
//!
//!         ok = i32;
//!
//!         err = enum {
//!             Overflow @ 1;
//!         }
//!     }
//! }
//! ```
//!
//! The Rust code, implementing both the calculator service itself and the tests:
//!
//! ```
//! use aldrin_client::{Handle, ObjectUuid};
//! use aldrin_test::TestBroker;
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
//!     // Create a broker:
//!     let mut broker = TestBroker::new();
//!
//!     // Run the broker:
//!     let broker_join = tokio::spawn(broker.take_broker().run());
//!
//!     // Create a client and run it:
//!     let mut client = broker.add_client().await;
//!     tokio::spawn(client.take_connection().run());
//!     let client_join = tokio::spawn(client.take_client().run());
//!
//!     // Run our calculator server:
//!     tokio::spawn(run_calculator(client.clone()));
//!
//!     // Create a proxy to the calculator:
//!     let service_id = client.wait_for_service(CALCULATOR_UUID, None).await?;
//!     let calc = CalculatorProxy::bind(client.clone(), service_id)?;
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
//!     client.shutdown();
//!     broker.shutdown_idle();
//!     broker_join.await??;
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
//!     while let Some(CalculatorFunction::Add(args, reply)) = calc.next().await {
//!         match args.lhs.checked_add(args.rhs) {
//!             Some(res) => reply.ok(res)?,
//!             None => reply.err(CalculatorAddError::Overflow)?,
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

#![deny(intra_doc_link_resolution_failure)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

#[cfg(test)]
mod test;
mod transport;

#[cfg(feature = "tokio")]
pub mod tokio_based;

use aldrin_broker::{Broker, BrokerHandle, Connection, ConnectionHandle};
use aldrin_client::{Client, Handle};
use aldrin_util::channel;
use futures_util::future;
use std::num::NonZeroUsize;
use std::ops::Deref;

pub use transport::TestTransport;

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
    /// Handle to the broker.
    pub handle: BrokerHandle,

    broker: Option<Broker>,
}

impl TestBroker {
    /// Creates a new broker with the default settings.
    ///
    /// This function uses the same defaults as [`aldrin_broker::Broker::new`].
    pub fn new() -> Self {
        let broker = Broker::new();
        let handle = broker.handle().clone();
        TestBroker {
            handle,
            broker: Some(broker),
        }
    }

    /// Creates a new broker with a custom fifo size.
    ///
    /// The `fifo_size` argument is passed directly to
    /// [`aldrin_broker::Broker::with_fifo_size`]. See the documentation there for more information.
    pub fn with_fifo_size(fifo_size: Option<NonZeroUsize>) -> Self {
        let broker = Broker::with_fifo_size(fifo_size);
        let handle = broker.handle().clone();
        TestBroker {
            handle,
            broker: Some(broker),
        }
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

    /// Creates a new `ClientBuilder`.
    ///
    /// Use of this function is recommended only if non-default settings are required for the
    /// [`Client`] or its [`Connection`]. A default [`Client`] can be added directly with
    /// [`add_client`](TestBroker::add_client).
    pub fn client_builder(&self) -> ClientBuilder {
        ClientBuilder {
            broker: &self.handle,
            channel: None,
            conn: None,
        }
    }

    /// Creates a default `Client`.
    ///
    /// If you need more control over the [`Client`] and [`Connection`] settings, then use
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

/// Builder struct for a new `Client`.
///
/// A [`ClientBuilder`] allows for more control over how [`Client`] and [`Connection`] are setup,
/// specifically the respective fifo sizes. If you do not any special settings though, it is
/// recommended to use [`TestBroker::add_client`] instead.
#[derive(Debug, Copy, Clone)]
pub struct ClientBuilder<'a> {
    broker: &'a BrokerHandle,
    channel: Option<usize>,
    conn: Option<usize>,
}

impl<'a> ClientBuilder<'a> {
    /// Creates a new `ClientBuilder`.
    ///
    /// The defaults after creating the [`ClientBuilder`] use an unbounded channel between
    /// [`Broker`] and [`Client`] and the a default fifo size for the [`Connection`] (see
    /// [`aldrin_broker::BrokerHandle::add_connection`]).
    pub fn new(broker: &'a BrokerHandle) -> Self {
        ClientBuilder {
            broker,
            channel: None,
            conn: None,
        }
    }

    /// Creates a new `TestClient` with the current settings.
    pub async fn build(self) -> TestClient {
        let (t1, t2): (Box<dyn TestTransport>, Box<dyn TestTransport>) = match self.channel {
            Some(fifo_size) => {
                let (t1, t2) = channel::bounded(fifo_size);
                (Box::new(t1), Box::new(t2))
            }
            None => {
                let (t1, t2) = channel::unbounded();
                (Box::new(t1), Box::new(t2))
            }
        };

        let client = Client::connect(t1);
        let conn = async {
            match self.conn.map(NonZeroUsize::new) {
                Some(Some(fifo_size)) => {
                    self.broker
                        .add_connection_with_fifo_size(t2, Some(fifo_size))
                        .await
                }
                Some(None) => self.broker.add_connection_with_fifo_size(t2, None).await,
                None => self.broker.add_connection(t2).await,
            }
        };

        let (client, conn) = future::join(client, conn).await;
        let client = client.expect("client failed to connect");
        let handle = client.handle().clone();
        let conn = conn.expect("connection failed to establish");
        let connection_handle = conn.handle().clone();

        TestClient {
            handle,
            connection_handle,
            client: Some(client),
            conn: Some(conn),
        }
    }

    /// Uses an unbounded channel as the transport between `Broker` and `Client`.
    ///
    /// This is the default after creating a new [`ClientBuilder`].
    pub fn unbounded_channel(mut self) -> Self {
        self.channel = None;
        self
    }

    /// Uses a bounded channel as the transport between `Broker` and `Client`.
    ///
    /// See [`aldrin_util::channel::bounded`] for more information on the `fifo_size` parameter.
    pub fn bounded_channel(mut self, fifo_size: usize) -> Self {
        self.channel = Some(fifo_size);
        self
    }

    /// Uses the default fifo size for the `Connection`.
    ///
    /// This is the default after creating a new [`ClientBuilder`]. See
    /// [`aldrin_broker::BrokerHandle::add_connection`] for more information on the default.
    pub fn default_connection_fifo_size(mut self) -> Self {
        self.conn = None;
        self
    }

    /// Uses an unbounded fifo for the `Connection`.
    ///
    /// This is equivalent to calling [`connection_fifo_size`](ClientBuilder::connection_fifo_size)
    /// with a `fifo_size` of 0. See [`aldrin_broker::BrokerHandle::add_connection_with_fifo_size`]
    /// for more information.
    pub fn unbounded_connection_fifo_size(mut self) -> Self {
        self.conn = Some(0);
        self
    }

    /// Uses a specific fifo size for the `Connection`.
    ///
    /// Calling this function with a `fifo_size` of 0 is equivalent to calling
    /// [`unbounded_connection_fifo_size`](ClientBuilder::unbounded_connection_fifo_size). See
    /// [`aldrin_broker::BrokerHandle::add_connection_with_fifo_size`] for more information.
    pub fn connection_fifo_size(mut self, fifo_size: usize) -> Self {
        self.conn = Some(fifo_size);
        self
    }
}

/// Client for use in tests.
///
/// [`TestClient`s](TestClient) with default settings can be created using
/// [`TestBroker::add_client`], or alternatively with a [`ClientBuilder`] if more control over the
/// settings is required.
///
/// After creating a [`TestClient`], it is fully connected to the [`TestBroker`], but neither the
/// underlying [`Client`] nor the [`Connection`] are running. This must be done manually by taking
/// both parts out and calling their respective `run` methods (see
/// [`take_client`](Self::take_client) and [`take_connection`](Self::take_connection).
///
/// [`TestClient`] dereferences to [`aldrin_client::Handle`] and thus all methods on [`Handle`] can
/// be called on [`TestClient`] as well.
#[derive(Debug)]
pub struct TestClient {
    /// `Handle` to the `Client`.
    pub handle: Handle,

    /// `ConnectionHandle` to the `Connection`.
    pub connection_handle: ConnectionHandle,

    client: Option<Client<Box<dyn TestTransport>>>,
    conn: Option<Connection<Box<dyn TestTransport>>>,
}

impl TestClient {
    /// Creates a new `ClientBuilder`.
    pub fn builder(broker: &BrokerHandle) -> ClientBuilder {
        ClientBuilder::new(broker)
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
    pub fn take_client(&mut self) -> Client<Box<dyn TestTransport>> {
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
    pub fn take_connection(&mut self) -> Connection<Box<dyn TestTransport>> {
        self.conn.take().expect("connection already taken")
    }
}

impl Deref for TestClient {
    type Target = Handle;

    fn deref(&self) -> &Handle {
        &self.handle
    }
}
