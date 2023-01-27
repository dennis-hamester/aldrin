use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use aldrin_proto::message::{ChannelEnd, CloseChannelEndResult};
use aldrin_proto::ChannelCookie;
use anyhow::{anyhow, Result};

const NAME: &str = "close-invalid-receiver";
const SHORT: &str = "Try to close a non-existent receiver";
const MESSAGE_TYPES: &[MessageType] = &[
    MessageType::CloseChannelEnd,
    MessageType::CloseChannelEndReply,
];

pub fn make_test() -> BrokerTest {
    BrokerTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(broker: &mut BrokerUnderTest) -> Result<()> {
    let mut client = broker.connect_client().await?;

    let result = client
        .send_close_channel_end(0, ChannelCookie::new_v4(), ChannelEnd::Receiver)
        .await?;

    if result == CloseChannelEndResult::InvalidChannel {
        Ok(())
    } else {
        Err(anyhow!(
            "close-channel-end-reply received with result {result:?} but expected {:?}",
            CloseChannelEndResult::InvalidChannel,
        ))
    }
}
