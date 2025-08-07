use super::BrokerShutdown;
use crate::conn::ConnectionEvent;
use crate::conn_id::ConnectionIdManager;
#[cfg(feature = "statistics")]
use crate::BrokerStatistics;
use crate::{AcceptError, Acceptor, Connection, ConnectionHandle};
use aldrin_core::transport::{AsyncTransport, Buffered};
use aldrin_core::ProtocolVersion;
use futures_channel::mpsc;
#[cfg(feature = "statistics")]
use futures_channel::oneshot;
use futures_util::sink::SinkExt;

/// Handle of an active broker.
///
/// `BrokerHandle`s are used to interact with an active [`Broker`](crate::Broker). The first
/// `BrokerHandle` can be acquired from the `Broker` with [`handle`](crate::Broker::handle), and
/// from then on, `BrokerHandle`s can be cloned cheaply.
///
/// The `Broker` will automatically shut down when the last `BrokerHandle` has been dropped and
/// while there are no active clients.
#[derive(Debug, Clone)]
pub struct BrokerHandle {
    send: mpsc::Sender<ConnectionEvent>,
    ids: ConnectionIdManager,
}

impl BrokerHandle {
    pub(crate) fn new(send: mpsc::Sender<ConnectionEvent>) -> Self {
        Self {
            send,
            ids: ConnectionIdManager::new(),
        }
    }

    pub(crate) async fn add_connection<T: AsyncTransport + Unpin>(
        &mut self,
        transport: Buffered<T>,
        version: ProtocolVersion,
    ) -> Result<Connection<T>, BrokerShutdown> {
        let id = self.ids.acquire();
        let (send, recv) = mpsc::unbounded();

        self.send
            .send(ConnectionEvent::NewConnection(id.clone(), version, send))
            .await
            .map_err(|_| BrokerShutdown)?;

        Ok(Connection::new(
            transport,
            version,
            id,
            self.send.clone(),
            recv,
        ))
    }

    /// Establishes a new connection.
    ///
    /// This method performs the initial connection setup and Aldrin handshake between broker and
    /// client. If successful, the resulting [`Connection`] must be [`run`](Connection::run) and
    /// polled to completion, much like the [`Broker`](crate::Broker) itself.
    ///
    /// The Aldrin protocol allows client and broker to exchange custom user data during the
    /// handshake. This function will ignore the client's user data. If you need to inspect the data
    /// and possibly reject some clients, then use [`Acceptor`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create an AsyncTransport to a new incoming connection:
    /// // let t = ...
    ///
    /// # let mut broker_handle = TestBroker::new();
    /// # let (t, t2) = aldrin_broker::core::channel::unbounded();
    /// # let client_join = tokio::spawn(aldrin::Client::connect(t2));
    /// // Establish a connection to the client:
    /// let connection = broker_handle.connect(t).await?;
    ///
    /// // Run the connection:
    /// tokio::spawn(connection.run());
    ///
    /// // The connection is now active and the client is fully connected.
    /// # let client = client_join.await??;
    /// # tokio::spawn(client.run());
    /// # broker_handle.join().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect<T>(&mut self, transport: T) -> Result<Connection<T>, AcceptError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        Acceptor::new(transport).await?.accept(self).await
    }

    /// Shuts down the broker.
    ///
    /// This method informs the [`Broker`](crate::Broker) that it should initiate shutdown, but
    /// doesn't block until `Broker` has done so. The `Broker` will cleanly shut down all
    /// connections, before the [`Broker::run`](crate::Broker::run) returns.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_broker::Broker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let broker = Broker::new();
    /// let mut handle = broker.handle().clone();
    /// let join = tokio::spawn(broker.run());
    ///
    /// // Tell the broker to shutdown:
    /// handle.shutdown().await;
    ///
    /// // `run` will return as soon as all connections have been shut down:
    /// join.await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&mut self) {
        let _ = self.send.send(ConnectionEvent::ShutdownBroker).await;
    }

    /// Shuts down the broker when the last client disconnects.
    ///
    /// This method informs the [`Broker`](crate::Broker) that it should shutdown as soon as there
    /// are no active clients left.
    ///
    /// Calling this method does not prevent new connections. It also doesn't actively tell the
    /// connected clients to shut down.
    ///
    /// # Examples
    ///
    /// ```
    /// use aldrin_broker::Broker;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let broker = Broker::new();
    /// let mut handle = broker.handle().clone();
    /// let join = tokio::spawn(broker.run());
    ///
    /// // Tell the broker to shutdown when it becomes idle:
    /// handle.shutdown_idle().await;
    ///
    /// // `run` will return as soon as the last client disconnects:
    /// join.await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown_idle(&mut self) {
        let _ = self.send.send(ConnectionEvent::ShutdownIdleBroker).await;
    }

    /// Shuts down a specific connection.
    ///
    /// Similar to the other shutdown methods, this method will only initiate shutdown of the
    /// [`Connection`] specified by `conn` and then return before it has actually shut down.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create an AsyncTransport to a new incoming connection:
    /// // let t = ...
    ///
    /// # let mut broker_handle = TestBroker::new();
    /// # let (t, t2) = aldrin_broker::core::channel::unbounded();
    /// # let client_join = tokio::spawn(aldrin::Client::connect(t2));
    /// // Establish a connection to the client:
    /// let connection = broker_handle.connect(t).await?;
    ///
    /// // Get a handle to the connection:
    /// let connection_handle = connection.handle().clone();
    ///
    /// // Run the connection:
    /// let connection_join = tokio::spawn(connection.run());
    /// # let client = client_join.await??;
    /// # tokio::spawn(client.run());
    ///
    /// // Tell the broker to shut down the connection again:
    /// broker_handle.shutdown_connection(&connection_handle).await?;
    /// connection_join.await??;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown_connection(
        &mut self,
        conn: &ConnectionHandle,
    ) -> Result<(), BrokerShutdown> {
        self.send
            .send(ConnectionEvent::ShutdownConnection(conn.id().clone()))
            .await
            .map_err(|_| BrokerShutdown)
    }

    /// Gets the current broker statistics.
    ///
    /// Some statistics are measured over the time interval between two calls to this function. Such
    /// statistics will be reset to 0 when this function is called.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker_handle = TestBroker::new();
    /// let statistics = broker_handle.take_statistics().await?;
    ///
    /// // Calculate the duration over which the statistics were measured.
    /// let time_diff = statistics.end() - statistics.start();
    ///
    /// println!("The current number of connections is {}.", statistics.num_connections());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "statistics")]
    #[cfg_attr(docsrs, doc(cfg(feature = "statistics")))]
    pub async fn take_statistics(&mut self) -> Result<BrokerStatistics, BrokerShutdown> {
        let (send, recv) = oneshot::channel();
        self.send
            .send(ConnectionEvent::TakeStatistics(send))
            .await
            .map_err(|_| BrokerShutdown)?;
        recv.await.map_err(|_| BrokerShutdown)
    }
}
