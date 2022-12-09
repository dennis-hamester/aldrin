#![no_main]

use aldrin_proto::message::Packetizer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Vec<u8>>| {
    let mut packetizer = Packetizer::new();

    for data in data {
        packetizer.extend_from_slice(&data);
        while let Some(_) = packetizer.next_message() {}
    }
});
