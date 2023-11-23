#![no_main]

use aldrin_core::message::{Message, MessageOps};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|msg: Message| {
    let _ = msg.serialize_message();
});
