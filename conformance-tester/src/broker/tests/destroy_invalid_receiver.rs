use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use aldrin_proto::message::{ChannelEnd, DestroyChannelEndResult};
use aldrin_proto::ChannelCookie;
use anyhow::{anyhow, Result};

const NAME: &str = "destroy-invalid-receiver";
const SHORT: &str = "Try to destroy a non-existent receiver";
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
        .send_destroy_channel_end(0, ChannelCookie::new_v4(), ChannelEnd::Receiver)
        .await?;

    if result == DestroyChannelEndResult::InvalidChannel {
        Ok(())
    } else {
        Err(anyhow!(
            "destroy-channel-end-reply received with result {result:?} but expected {:?}",
            DestroyChannelEndResult::InvalidChannel,
        ))
    }
}
