use super::{BrokerTest, BrokerUnderTest};
use crate::test::MessageType;
use aldrin_proto::message::{ChannelEnd, ChannelEndClosed, Message};
use anyhow::{anyhow, Context, Result};

const NAME: &str = "send-item-to-unclaimed-receiver-closes-channel";
const SHORT: &str = "Channels are closed when items are sent while the receiver is unclaimed";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::SendItem];

pub fn make_test() -> BrokerTest {
    BrokerTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(broker: &mut BrokerUnderTest) -> Result<()> {
    let mut client = broker.connect_client().await?;
    let cookie = client.create_channel(0, ChannelEnd::Sender).await?;

    // Send an item even though the channel hasn't been established yet.
    client
        .send_item(cookie, &())
        .await
        .with_context(|| anyhow!("failed to send send-item message"))?;

    let msg = client
        .receive()
        .await
        .with_context(|| anyhow!("failed to receive channel-end-closed message"))?;
    let expected = Message::ChannelEndClosed(ChannelEndClosed {
        cookie,
        end: ChannelEnd::Receiver,
    });
    if msg != expected {
        return Err(anyhow!("expected {expected:?}, but got {msg:?}"));
    }

    Ok(())
}
