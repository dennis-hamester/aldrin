#![no_main]

use aldrin_core::message::{Message, MessageOps};
use bytes::BytesMut;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let data = BytesMut::from(data);
    let _ = Message::deserialize_message(data);
});
