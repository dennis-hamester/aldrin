use super::BrokerShutdown;
#[cfg(feature = "statistics")]
use super::BrokerStatistics;
use crate::conn::{Connection, ConnectionEvent, ConnectionHandle, EstablishError};
use crate::conn_id::ConnectionIdManager;
use aldrin_proto::{AsyncTransport, AsyncTransportExt, ConnectReply, Message, Value};
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
    pub(crate) fn new(send: mpsc::Sender<ConnectionEvent>) -> BrokerHandle {
        BrokerHandle {
            send,
            ids: ConnectionIdManager::new(),
        }
    }

    /// Establishes a new connection.
    ///
    /// This method performs the initial connection setup and Aldrin handshake between broker and
    /// client. If successful, the resulting [`Connection`] must be [`run`](Connection::run) and
    /// polled to completion, much like the [`Broker`](crate::Broker) itself.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create an AsyncTransport to a new incoming connection:
    /// // let t = ...
    ///
    /// # let mut broker_handle = TestBroker::new();
    /// # let (t, t2) = aldrin_channel::unbounded();
    /// # let client_join = tokio::spawn(aldrin_client::Client::connect(t2));
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
    pub async fn connect<T>(&mut self, mut t: T) -> Result<Connection<T>, EstablishError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        match t.receive().await? {
            Message::Connect(msg) if msg.version == aldrin_proto::VERSION => {
                t.send_and_flush(Message::ConnectReply(ConnectReply::Ok(Value::None)))
                    .await?;
                Ok(())
            }

            Message::Connect(msg) => {
                t.send_and_flush(Message::ConnectReply(ConnectReply::VersionMismatch(
                    aldrin_proto::VERSION,
                )))
                .await
                .ok();
                Err(EstablishError::VersionMismatch(msg.version))
            }

            msg => Err(EstablishError::UnexpectedMessageReceived(msg)),
        }?;

        let id = self.ids.acquire();
        let (send, recv) = mpsc::unbounded();

        self.send
            .send(ConnectionEvent::NewConnection(id.clone(), send))
            .await
            .map_err(|_| EstablishError::BrokerShutdown)?;

        Ok(Connection::new(t, id, self.send.clone(), recv))
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
        self.send.send(ConnectionEvent::ShutdownBroker).await.ok();
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
        self.send
            .send(ConnectionEvent::ShutdownIdleBroker)
            .await
            .ok();
    }

    /// Shuts down a specific connection.
    ///
    /// Similar to the other shutdown methods, this method will only initiate shutdown of the
    /// [`Connection`] specified by `conn` and then return before it has actually shut down.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create an AsyncTransport to a new incoming connection:
    /// // let t = ...
    ///
    /// # let mut broker_handle = TestBroker::new();
    /// # let (t, t2) = aldrin_channel::unbounded();
    /// # let client_join = tokio::spawn(aldrin_client::Client::connect(t2));
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
    /// # use aldrin_test::tokio_based::TestBroker;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut broker_handle = TestBroker::new();
    /// let statistics = broker_handle.take_statistics().await?;
    ///
    /// // Calculate the duration over which the statistics were measured.
    /// let time_diff = statistics.end - statistics.start;
    ///
    /// println!("The current number of connections is {}.", statistics.num_connections);
    /// println!(
    ///     "{} connections were added within the last {} seconds.",
    ///     statistics.connections_added,
    ///     time_diff.as_secs_f32()
    /// );
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
