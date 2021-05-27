use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use aldrin_proto::{Connect, ConnectReply, Message, VERSION};
use anyhow::{anyhow, Context, Result};

const NAME: &str = "connect-version-mismatch";
const SHORT: &str = "Unsuccessful two-way connection handshake";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::Connect, MessageType::ConnectReply];

pub fn make_test() -> BrokerTest {
    BrokerTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(broker: &mut BrokerUnderTest) -> Result<()> {
    let mut client = broker.connect_client_no_handshake().await?;

    client
        .send(Message::Connect(Connect {
            version: VERSION - 1,
        }))
        .await
        .with_context(|| anyhow!("failed to send connect message to broker"))?;

    let msg = client
        .receive()
        .await
        .with_context(|| anyhow!("failed to receive connect-reply from broker"))?;

    if let Message::ConnectReply(ConnectReply::VersionMismatch(_)) = msg {
        Ok(())
    } else {
        Err(anyhow!(
            "expected connect-reply with version-mismatch, but got {}",
            serde_json::to_string(&msg).unwrap()
        ))
    }
}
