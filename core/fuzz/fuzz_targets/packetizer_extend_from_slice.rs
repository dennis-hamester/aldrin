#![no_main]

use aldrin_core::message::Packetizer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Vec<u8>>| {
    let mut packetizer = Packetizer::new();
    let mut msgs = Vec::new();

    for data in data {
        packetizer.extend_from_slice(&data);
        while let Some(msg) = packetizer.next_message() {
            msgs.push(msg);
        }
    }
});
