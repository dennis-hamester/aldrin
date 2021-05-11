use super::{ClientTest, ClientUnderTest};
use crate::test::MessageType;
use aldrin_proto::Message;
use anyhow::{anyhow, Context, Result};

const NAME: &str = "shutdown-by-broker";
const SHORT: &str = "Shutdown initiated by the broker";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::Shutdown];

pub fn make_test() -> ClientTest {
    ClientTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(client: &mut ClientUnderTest) -> Result<()> {
    client.connect().await?;

    client
        .send_message(Message::Shutdown(()))
        .await
        .with_context(|| anyhow!("failed to send shutdown message"))?;

    let msg = client
        .recv_message()
        .await
        .with_context(|| anyhow!("failed to receive connect message"))?;

    match msg {
        Message::Shutdown(()) => Ok(()),
        _ => Err(anyhow!(
            "expected connect message but received {}",
            serde_json::to_string(&msg).unwrap()
        )),
    }
}
