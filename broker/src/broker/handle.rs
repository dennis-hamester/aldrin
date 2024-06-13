use super::BrokerShutdown;
#[cfg(feature = "statistics")]
use super::BrokerStatistics;
use crate::conn::{Connection, ConnectionEvent, ConnectionHandle, EstablishError};
use crate::conn_id::ConnectionIdManager;
use crate::core::message::{ConnectData, ConnectReply, ConnectReply2, ConnectReplyData, Message};
use crate::core::transport::{AsyncTransport, AsyncTransportExt};
use crate::core::{
    Deserialize, DeserializeError, ProtocolVersion, Serialize, SerializedValue,
    SerializedValueSlice,
};
use futures_channel::mpsc;
#[cfg(feature = "statistics")]
use futures_channel::oneshot;
use futures_util::sink::SinkExt;

const PROTOCOL_VERSION_MIN: ProtocolVersion = ProtocolVersion::V1_14;
const PROTOCOL_VERSION_MAX: ProtocolVersion = ProtocolVersion::V1_16;

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
    /// The Aldrin protocol allows client and broker to exchange custom user data during the
    /// handshake. This function will ignore the client's user data. If you need to inspect the data
    /// and possibly reject some clients, then use [`begin_connect`](Self::begin_connect).
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
    pub async fn connect<T>(&mut self, t: T) -> Result<Connection<T>, EstablishError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        self.begin_connect(t).await?.accept(None).await
    }

    /// Begins establishing a new connection.
    ///
    /// Unlike [`connect`](Self::connect), this function will not automatically establish a
    /// connection. It will only receive the client's initial connection message. This allows you to
    /// inspect the client's user data and accept or reject the client.
    pub async fn begin_connect<T>(
        &mut self,
        mut t: T,
    ) -> Result<PendingConnection<T>, EstablishError<T::Error>>
    where
        T: AsyncTransport + Unpin,
    {
        let (connect2, data, major_version, minor_version) =
            match t.receive().await.map_err(EstablishError::Transport)? {
                Message::Connect(msg) => {
                    let data = ConnectData {
                        user: Some(msg.value),
                    };

                    (false, data, ProtocolVersion::MAJOR, msg.version)
                }

                Message::Connect2(msg) => {
                    let data = msg.deserialize_connect_data()?;
                    (true, data, msg.major_version, msg.minor_version)
                }

                msg => return Err(EstablishError::UnexpectedMessageReceived(msg)),
            };

        let version = match select_protocol_version(major_version, minor_version, connect2) {
            Some(version) => version,

            None => {
                if connect2 {
                    let _ = t
                        .send_and_flush(Message::ConnectReply2(
                            ConnectReply2::incompatible_version_with_serialize_data(
                                &ConnectReplyData::new(),
                            )?,
                        ))
                        .await;
                } else {
                    let _ = t
                        .send_and_flush(Message::ConnectReply(ConnectReply::IncompatibleVersion(
                            14,
                        )))
                        .await;
                }

                return Err(EstablishError::IncompatibleVersion);
            }
        };

        Ok(PendingConnection::new(
            self.clone(),
            t,
            connect2,
            data,
            version,
        ))
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
    /// println!(
    ///     "{} connections were added within the last {} seconds.",
    ///     statistics.connections_added(),
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

/// A pending client connection, that hasn't been accepted or rejected yet.
///
/// This type is acquired by [`BrokerHandle::begin_connect`]. It allows inspection of the [client's
/// user data](Self::user_data) and to [accept](Self::accept) or [reject](Self::reject) a client.
///
/// Dropping this type will simply also drop the transport. No message will be sent back to the
/// client in this case.
#[derive(Debug)]
pub struct PendingConnection<T: AsyncTransport + Unpin> {
    handle: BrokerHandle,
    t: T,
    connect2: bool,
    data: ConnectData,
    version: ProtocolVersion,
}

impl<T: AsyncTransport + Unpin> PendingConnection<T> {
    fn new(
        handle: BrokerHandle,
        t: T,
        connect2: bool,
        data: ConnectData,
        version: ProtocolVersion,
    ) -> Self {
        Self {
            handle,
            t,
            connect2,
            data,
            version,
        }
    }

    /// Returns the client's user data.
    pub fn user_data(&self) -> Option<&SerializedValueSlice> {
        self.data.user.as_deref()
    }

    /// Deserializes the client's user data.
    pub fn deserialize_client_data<D: Deserialize>(&self) -> Option<Result<D, DeserializeError>> {
        self.data.deserialize_user()
    }

    /// Returns the selected protocol version for this connection.
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.version
    }

    /// Accepts a client with optional user data.
    ///
    /// The resulting [`Connection`] must be [`run`](Connection::run) and polled to completion, much
    /// like the [`Broker`](crate::Broker) itself.
    pub async fn accept(
        mut self,
        user_data: Option<SerializedValue>,
    ) -> Result<Connection<T>, EstablishError<T::Error>> {
        if self.connect2 {
            self.t
                .send_and_flush(Message::ConnectReply2(
                    ConnectReply2::ok_with_serialize_data(
                        self.version.minor(),
                        &ConnectReplyData { user: user_data },
                    )?,
                ))
                .await
                .map_err(EstablishError::Transport)?;
        } else {
            let user_data = user_data
                .map(Ok)
                .unwrap_or_else(|| SerializedValue::serialize(&()))?;

            self.t
                .send_and_flush(Message::ConnectReply(ConnectReply::Ok(user_data)))
                .await
                .map_err(EstablishError::Transport)?;
        }

        let id = self.handle.ids.acquire();
        let (send, recv) = mpsc::unbounded();

        self.handle
            .send
            .send(ConnectionEvent::NewConnection(
                id.clone(),
                self.version,
                send,
            ))
            .await
            .map_err(|_| EstablishError::Shutdown)?;

        let conn = Connection::new(self.t, id, self.handle.send, recv);

        Ok(conn)
    }

    /// Accepts a client with optional user data.
    ///
    /// The resulting [`Connection`] must be [`run`](Connection::run) and polled to completion, much
    /// like the [`Broker`](crate::Broker) itself.
    pub async fn accept_serialize<D: Serialize + ?Sized>(
        self,
        user_data: Option<&D>,
    ) -> Result<Connection<T>, EstablishError<T::Error>> {
        let user_data = user_data.map(SerializedValue::serialize).transpose()?;
        self.accept(user_data).await
    }

    /// Rejects a client with optional user data.
    pub async fn reject(
        mut self,
        user_data: Option<SerializedValue>,
    ) -> Result<(), EstablishError<T::Error>> {
        if self.connect2 {
            self.t
                .send_and_flush(Message::ConnectReply2(
                    ConnectReply2::rejected_with_serialize_data(&ConnectReplyData {
                        user: user_data,
                    })?,
                ))
                .await
                .map_err(EstablishError::Transport)?;
        } else {
            let user_data = user_data
                .map(Ok)
                .unwrap_or_else(|| SerializedValue::serialize(&()))?;

            self.t
                .send_and_flush(Message::ConnectReply(ConnectReply::Rejected(user_data)))
                .await
                .map_err(EstablishError::Transport)?;
        }

        Ok(())
    }

    /// Rejects a client with optional user data.
    pub async fn reject_serialize<D: Serialize + ?Sized>(
        self,
        user_data: Option<&D>,
    ) -> Result<(), EstablishError<T::Error>> {
        let user_data = user_data.map(SerializedValue::serialize).transpose()?;
        self.reject(user_data).await
    }
}

fn select_protocol_version(major: u32, minor: u32, connect2: bool) -> Option<ProtocolVersion> {
    if connect2 {
        if (major == ProtocolVersion::MAJOR) && (minor >= PROTOCOL_VERSION_MIN.minor()) {
            let minor = minor.min(PROTOCOL_VERSION_MAX.minor());
            Some(ProtocolVersion::new(ProtocolVersion::MAJOR, minor).unwrap())
        } else {
            None
        }
    } else if (major == ProtocolVersion::V1_14.major()) && (minor == ProtocolVersion::V1_14.minor())
    {
        Some(ProtocolVersion::V1_14)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::select_protocol_version;
    use aldrin::core::ProtocolVersion;

    #[test]
    fn test_select_protocol_version() {
        assert_eq!(
            select_protocol_version(1, 14, true),
            Some(ProtocolVersion::V1_14)
        );
        assert_eq!(
            select_protocol_version(1, 15, true),
            Some(ProtocolVersion::V1_15)
        );
        assert_eq!(
            select_protocol_version(1, 16, true),
            Some(ProtocolVersion::V1_16)
        );
        assert_eq!(
            select_protocol_version(1, 17, true),
            Some(ProtocolVersion::V1_16)
        );
        assert_eq!(select_protocol_version(1, 13, true), None);
        assert_eq!(select_protocol_version(2, 0, true), None);
        assert_eq!(select_protocol_version(2, 14, true), None);

        assert_eq!(
            select_protocol_version(1, 14, false),
            Some(ProtocolVersion::V1_14)
        );
        assert_eq!(select_protocol_version(1, 15, false), None);
        assert_eq!(select_protocol_version(1, 13, false), None);
        assert_eq!(select_protocol_version(2, 0, false), None);
        assert_eq!(select_protocol_version(2, 14, false), None);
    }
}
