use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use aldrin_proto::ChannelEnd;
use anyhow::Result;

const NAME: &str = "create-channel-with-claimed-receiver";
const SHORT: &str = "Creates a channel with a claimed receiver";
const MESSAGE_TYPES: &[MessageType] =
    &[MessageType::CreateChannel, MessageType::CreateChannelReply];

pub fn make_test() -> BrokerTest {
    BrokerTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(broker: &mut BrokerUnderTest) -> Result<()> {
    let mut client = broker.connect_client().await?;
    client.create_channel(0, ChannelEnd::Receiver).await?;
    Ok(())
}
