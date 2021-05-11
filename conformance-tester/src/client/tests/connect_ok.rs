use super::{ClientTest, ClientUnderTest};
use crate::test::MessageType;
use anyhow::Result;

const NAME: &str = "connect-ok";
const SHORT: &str = "Successful two-way connection handshake";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::Connect, MessageType::ConnectReply];

pub fn make_test() -> ClientTest {
    ClientTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(client: &mut ClientUnderTest) -> Result<()> {
    client.connect().await?;
    Ok(())
}
