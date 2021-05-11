use super::{ClientTest, ClientUnderTest};
use crate::test::MessageType;
use aldrin_conformance_test_shared::client::ToClientMessage;
use aldrin_proto::Message;
use anyhow::{anyhow, Context, Result};

const NAME: &str = "shutdown-by-client";
const SHORT: &str = "Shutdown initiated by the client";
const MESSAGE_TYPES: &[MessageType] = &[MessageType::Shutdown];

pub fn make_test() -> ClientTest {
    ClientTest::new(NAME, SHORT, None, MESSAGE_TYPES, run)
}

async fn run(client: &mut ClientUnderTest) -> Result<()> {
    client.connect().await?;

    client
        .send_stdin(ToClientMessage::Shutdown(()))
        .await
        .with_context(|| anyhow!("failed to write shutdown message to client's stdin"))?;

    let msg = client
        .recv_message()
        .await
        .with_context(|| anyhow!("failed to receive connect message"))?;

    match msg {
        Message::Shutdown(()) => {}
        _ => {
            return Err(anyhow!(
                "expected connect message but received {}",
                serde_json::to_string(&msg).unwrap()
            ));
        }
    }

    client
        .send_message(Message::Shutdown(()))
        .await
        .with_context(|| anyhow!("failed to send shutdown message"))?;

    Ok(())
}
