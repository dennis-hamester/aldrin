use super::{ClientTest, ClientUnderTest};
use crate::test::MessageType;
use aldrin_proto::message::{Connect, ConnectReply, Message};
use anyhow::{anyhow, Context, Result};

const NAME: &str = "connect-version-mismatch";
const SHORT: &str = "Unsuccessful two-way connection handshake";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::Connect, MessageType::ConnectReply];

pub fn make_test() -> ClientTest {
    ClientTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(client: &mut ClientUnderTest) -> Result<()> {
    let msg = client
        .recv_message()
        .await
        .with_context(|| anyhow!("failed to receive connect message"))?;

    if let Message::Connect(Connect { version, .. }) = msg {
        let version = version.wrapping_add(1);
        client
            .send_message(Message::ConnectReply(ConnectReply::VersionMismatch(
                version,
            )))
            .await
            .with_context(|| anyhow!("failed to send connect-reply message"))?;
        Ok(())
    } else {
        Err(anyhow!("expected connect message but received {msg:?}"))
    }
}
