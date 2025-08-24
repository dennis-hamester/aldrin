#[cfg(test)]
mod test;

use crate::{BrokerHandle, BrokerShutdown, Connection};
use aldrin_core::message::{
    ConnectData, ConnectReply, ConnectReply2, ConnectReplyData, ConnectResult, Message, MessageOps,
};
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::transport::{AsyncTransport, AsyncTransportExt, Buffered};
use aldrin_core::{
    Deserialize, DeserializeError, ProtocolVersion, Serialize, SerializeError, SerializedValue,
    SerializedValueSlice, ValueConversionError,
};
use thiserror::Error;

/// Accepts or rejects new connections.
#[derive(Debug)]
pub struct Acceptor<T> {
    transport: Buffered<T>,
    connect2: bool,
    version: ProtocolVersion,
    data: ConnectData,
    reply_data: ConnectReplyData,
}

impl<T: AsyncTransport + Unpin> Acceptor<T> {
    /// Creates a new [`Acceptor`].
    pub async fn new(transport: T) -> Result<Self, AcceptError<T::Error>> {
        let mut transport = transport.buffered();

        let (connect2, data, version) =
            match transport.receive().await.map_err(AcceptError::Transport)? {
                Message::Connect(msg) => {
                    let data = ConnectData {
                        user: Some(msg.value),
                    };

                    (false, data, ProtocolVersion::new(1, msg.version))
                }

                Message::Connect2(msg) => {
                    let data = msg.value.deserialize()?;
                    let version = ProtocolVersion::new(msg.major_version, msg.minor_version);

                    (true, data, version)
                }

                msg => return Err(AcceptError::UnexpectedMessageReceived(msg)),
            };

        let Some(version) = select_protocol_version(version, connect2) else {
            if connect2 {
                let msg = ConnectReply2 {
                    result: ConnectResult::IncompatibleVersion,
                    value: SerializedValue::serialize(ConnectReplyData::new())?,
                };

                let _ = send(transport, msg, ProtocolVersion::V1_14).await;
            } else {
                let _ = send(
                    transport,
                    ConnectReply::IncompatibleVersion(14),
                    ProtocolVersion::V1_14,
                )
                .await;
            }

            return Err(AcceptError::IncompatibleVersion(version));
        };

        Ok(Self {
            transport,
            connect2,
            version,
            data,
            reply_data: ConnectReplyData::new(),
        })
    }

    /// Returns the [`ProtocolVersion`] this connection will use.
    pub fn version(&self) -> ProtocolVersion {
        self.version
    }

    /// Returns a reference to the client's data if any.
    pub fn client_data(&self) -> Option<&SerializedValueSlice> {
        self.data.user.as_deref()
    }

    /// Deserializes the client's data.
    pub fn deserialize_client_data_as<U: Tag, V: Deserialize<U>>(
        &self,
    ) -> Option<Result<V, DeserializeError>> {
        self.data.deserialize_user_as()
    }

    /// Deserializes the client's data.
    pub fn deserialize_client_data<U: PrimaryTag + Deserialize<U::Tag>>(
        &self,
    ) -> Option<Result<U, DeserializeError>> {
        self.data.deserialize_user()
    }

    /// Sets the data, that will be sent back to the client.
    pub fn set_reply_data(&mut self, data: SerializedValue) {
        self.reply_data.user = Some(data);
    }

    /// Sets the data, that will be sent back to the client, by serializing some value.
    pub fn serialize_reply_data_as<U: Tag>(
        &mut self,
        data: impl Serialize<U>,
    ) -> Result<(), SerializeError> {
        self.reply_data.serialize_user_as(data)?;
        Ok(())
    }

    /// Sets the data, that will be sent back to the client, by serializing some value.
    pub fn serialize_reply_data<U: PrimaryTag + Serialize<U::Tag>>(
        &mut self,
        data: U,
    ) -> Result<(), SerializeError> {
        self.reply_data.serialize_user(data)?;
        Ok(())
    }

    /// Accepts the connection and adds it to the given broker.
    pub async fn accept(
        mut self,
        broker: &mut BrokerHandle,
    ) -> Result<Connection<T>, AcceptError<T::Error>> {
        if self.connect2 {
            let msg = ConnectReply2 {
                result: ConnectResult::Ok(self.version.minor()),
                value: SerializedValue::serialize(self.reply_data)?,
            };

            send(&mut self.transport, msg, self.version).await?;
        } else {
            let user_data = self
                .reply_data
                .user
                .map(Ok)
                .unwrap_or_else(|| SerializedValue::serialize(()))?;

            let msg = ConnectReply::Ok(user_data);
            send(&mut self.transport, msg, self.version).await?;
        }

        broker
            .add_connection(self.transport, self.version)
            .await
            .map_err(Into::into)
    }

    /// Rejects the connection.
    pub async fn reject(self) -> Result<(), AcceptError<T::Error>> {
        if self.connect2 {
            let msg = ConnectReply2 {
                result: ConnectResult::Rejected,
                value: SerializedValue::serialize(self.reply_data)?,
            };

            send(self.transport, msg, self.version).await
        } else {
            let user_data = self
                .reply_data
                .user
                .map(Ok)
                .unwrap_or_else(|| SerializedValue::serialize(()))?;

            let msg = ConnectReply::Rejected(user_data);
            send(self.transport, msg, self.version).await
        }
    }
}

async fn send<T: AsyncTransport + Unpin>(
    mut transport: T,
    msg: impl Into<Message>,
    version: ProtocolVersion,
) -> Result<(), AcceptError<T::Error>> {
    let mut msg = msg.into();
    msg.convert_value(None, version)?;

    transport
        .send_and_flush(msg)
        .await
        .map_err(AcceptError::Transport)
}

/// Error while establishing a new connection.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AcceptError<T> {
    /// An unexpected message was received.
    #[error("unexpected message received")]
    UnexpectedMessageReceived(Message),

    /// The protocol version of the client is incompatible.
    #[error("incompatible protocol version {0}")]
    IncompatibleVersion(ProtocolVersion),

    /// The broker shut down.
    #[error("broker shut down")]
    Shutdown,

    /// The transport encountered an error.
    #[error(transparent)]
    Transport(T),

    /// A value failed to serialize.
    #[error(transparent)]
    Serialize(#[from] SerializeError),

    /// A value failed to deserialize.
    #[error(transparent)]
    Deserialize(#[from] DeserializeError),
}

impl<T> From<BrokerShutdown> for AcceptError<T> {
    fn from(_: BrokerShutdown) -> Self {
        Self::Shutdown
    }
}

impl<T> From<ValueConversionError> for AcceptError<T> {
    fn from(err: ValueConversionError) -> Self {
        match err {
            // Conversion here is always passed a valid version.
            ValueConversionError::InvalidVersion => unreachable!(),

            ValueConversionError::Serialize(e) => Self::Serialize(e),
            ValueConversionError::Deserialize(e) => Self::Deserialize(e),
        }
    }
}

fn select_protocol_version(version: ProtocolVersion, connect2: bool) -> Option<ProtocolVersion> {
    const MIN: ProtocolVersion = ProtocolVersion::V1_14;
    const MAX: ProtocolVersion = ProtocolVersion::V1_20;

    debug_assert!(MIN.major() == MAX.major());

    if version.major() != MIN.major() {
        None
    } else if connect2 {
        if version.minor() >= MIN.minor() {
            let minor = version.minor().min(MAX.minor());
            Some(ProtocolVersion::new(MIN.major(), minor))
        } else {
            None
        }
    } else if version.minor() == ProtocolVersion::V1_14.minor() {
        Some(ProtocolVersion::V1_14)
    } else {
        None
    }
}
