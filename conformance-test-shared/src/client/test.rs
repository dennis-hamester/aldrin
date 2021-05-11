use super::*;
use crate::test::test;
use serde_json::json;

#[test]
fn to_client_shutdown() {
    test(ToClientMessage::Shutdown(()), json!({ "shutdown": null }));
}
