use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use anyhow::Result;

const NAME: &str = "connect-ok";
const SHORT: &str = "Successful two-way connection handshake";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::Connect, MessageType::ConnectReply];

pub fn make_test() -> BrokerTest {
    BrokerTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(broker: &mut BrokerUnderTest) -> Result<()> {
    broker.connect_client().await?;
    Ok(())
}
