use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use aldrin_proto::{ChannelCookie, ChannelEnd, DestroyChannelEndResult};
use anyhow::{anyhow, Result};

const NAME: &str = "destroy-invalid-sender";
const SHORT: &str = "Try to destroy a non-existent sender";
const MESSAGE_TYPES: &[MessageType] = &[
    MessageType::DestroyChannelEnd,
    MessageType::DestroyChannelEndReply,
];

pub fn make_test() -> BrokerTest {
    BrokerTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(broker: &mut BrokerUnderTest) -> Result<()> {
    let mut client = broker.connect_client().await?;

    let result = client
        .send_destroy_channel_end(0, ChannelCookie::new_v4(), ChannelEnd::Sender)
        .await?;

    if result == DestroyChannelEndResult::InvalidChannel {
        Ok(())
    } else {
        Err(anyhow!(
            "destroy-channel-end-reply received with result {} but expected {}",
            serde_json::to_string(&result).unwrap(),
            serde_json::to_string(&DestroyChannelEndResult::InvalidChannel).unwrap()
        ))
    }
}
