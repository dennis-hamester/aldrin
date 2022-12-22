use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use aldrin_proto::message::{Message, Shutdown};
use anyhow::{anyhow, Context, Result};

const NAME: &str = "shutdown-by-client";
const SHORT: &str = "Shutdown initiated by the client";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::Shutdown];

pub fn make_test() -> BrokerTest {
    BrokerTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(broker: &mut BrokerUnderTest) -> Result<()> {
    let mut client = broker.connect_client().await?;

    client
        .send(Message::Shutdown(Shutdown))
        .await
        .with_context(|| anyhow!("failed to send shutdown message"))?;

    let msg = client
        .receive()
        .await
        .with_context(|| anyhow!("failed to receive shutdown message"))?;

    if let Message::Shutdown(Shutdown) = msg {
        Ok(())
    } else {
        Err(anyhow!("expected connect message but received {msg:?}"))
    }
}
