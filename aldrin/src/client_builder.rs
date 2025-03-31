#[cfg(test)]
mod test;

use crate::error::ConnectError;
use crate::Client;
use aldrin_core::message::{
    Connect, Connect2, ConnectData, ConnectReply, ConnectReplyData, ConnectResult, Message,
    MessageOps,
};
use aldrin_core::tags::{PrimaryTag, Tag};
use aldrin_core::transport::{AsyncTransport, AsyncTransportExt};
use aldrin_core::{ProtocolVersion, Serialize, SerializedValue};

/// Connects to a broker and constructs a [`Client`].
#[derive(Debug)]
pub struct ClientBuilder<T> {
    transport: T,
    data: Option<SerializedValue>,
}

impl<T: AsyncTransport + Unpin> ClientBuilder<T> {
    /// Creates a new [`ClientBuilder`].
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            data: None,
        }
    }

    /// Connects to the broker and returns the custom data it sent back.
    pub async fn connect_with_data(
        mut self,
    ) -> Result<(Client<T>, Option<SerializedValue>), ConnectError<T::Error>> {
        const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::V1_20;

        let connect_data = ConnectData { user: self.data };

        let mut connect = Connect2 {
            major_version: PROTOCOL_VERSION.major(),
            minor_version: PROTOCOL_VERSION.minor(),
            value: SerializedValue::serialize(connect_data)?,
        };

        connect.convert_value(None, ProtocolVersion::V1_14)?;

        self.transport
            .send_and_flush(connect)
            .await
            .map_err(ConnectError::Transport)?;

        let connect_reply = self
            .transport
            .receive()
            .await
            .map_err(ConnectError::Transport)?;

        let connect_reply = match connect_reply {
            Message::ConnectReply2(connect_reply) => connect_reply,
            msg => return Err(ConnectError::UnexpectedMessageReceived(msg)),
        };

        let connect_reply_data = connect_reply.value.deserialize::<ConnectReplyData>()?;

        let minor_version = match connect_reply.result {
            ConnectResult::Ok(minor_version) => minor_version,
            ConnectResult::Rejected => return Err(ConnectError::Rejected(connect_reply_data.user)),
            ConnectResult::IncompatibleVersion => return Err(ConnectError::IncompatibleVersion),
        };

        let version = ProtocolVersion::new(PROTOCOL_VERSION.major(), minor_version);
        if version > PROTOCOL_VERSION {
            return Err(ConnectError::IncompatibleVersion);
        }

        Ok((
            Client::new(self.transport, version),
            connect_reply_data.user,
        ))
    }

    /// Connects to the broker and discards the custom data it sent back.
    pub async fn connect(self) -> Result<Client<T>, ConnectError<T::Error>> {
        let (client, _) = self.connect_with_data().await?;
        Ok(client)
    }

    /// Connects to the broker using the old 1.14 protocol and returns the custom data it sent back.
    pub async fn connect1_with_data(
        mut self,
    ) -> Result<(Client<T>, SerializedValue), ConnectError<T::Error>> {
        const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::V1_14;

        let value = self
            .data
            .map(Ok)
            .unwrap_or_else(|| SerializedValue::serialize(()))?;

        let mut connect = Connect {
            version: PROTOCOL_VERSION.minor(),
            value,
        };

        connect.convert_value(None, PROTOCOL_VERSION)?;

        self.transport
            .send_and_flush(connect)
            .await
            .map_err(ConnectError::Transport)?;

        let connect_reply = self
            .transport
            .receive()
            .await
            .map_err(ConnectError::Transport)?;

        let connect_reply = match connect_reply {
            Message::ConnectReply(connect_reply) => connect_reply,
            msg => return Err(ConnectError::UnexpectedMessageReceived(msg)),
        };

        match connect_reply {
            ConnectReply::Ok(data) => Ok((Client::new(self.transport, PROTOCOL_VERSION), data)),
            ConnectReply::IncompatibleVersion(_) => Err(ConnectError::IncompatibleVersion),
            ConnectReply::Rejected(data) => Err(ConnectError::Rejected(Some(data))),
        }
    }

    /// Connects to the broker using the old 1.14 protocol and discards the custom data it sent
    /// back.
    pub async fn connect1(self) -> Result<Client<T>, ConnectError<T::Error>> {
        let (client, _) = self.connect1_with_data().await?;
        Ok(client)
    }

    /// Sets the data, that will be sent to the broker.
    pub fn set_data(&mut self, data: SerializedValue) {
        self.data = Some(data);
    }

    /// Sets the data, that will be sent to the broker, by serializing some value.
    pub fn serialize_data_as<U: Tag, V: Serialize<U>>(
        &mut self,
        data: V,
    ) -> Result<(), ConnectError<T::Error>> {
        self.data = SerializedValue::serialize_as(data).map(Some)?;
        Ok(())
    }

    /// Sets the data, that will be sent to the broker, by serializing some value.
    pub fn serialize_data<U: PrimaryTag + Serialize<U::Tag>>(
        &mut self,
        data: U,
    ) -> Result<(), ConnectError<T::Error>> {
        self.data = SerializedValue::serialize(data).map(Some)?;
        Ok(())
    }
}
