use super::*;
use crate::test::test;
use serde_json::json;

#[test]
fn to_broker_shutdown() {
    test(ToBrokerMessage::Shutdown(()), json!({ "shutdown": null }));
}

#[test]
fn from_broker_ready() {
    test(
        FromBrokerMessage::Ready(FromBrokerReady { port: 12345 }),
        json!({ "ready": { "port": 12345 }}),
    );
}
