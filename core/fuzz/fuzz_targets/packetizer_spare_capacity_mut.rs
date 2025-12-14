#![no_main]

use aldrin_core::message::Packetizer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Vec<u8>>| {
    let mut packetizer = Packetizer::new();
    let mut msgs = Vec::new();

    for data in data {
        let len = data.len();
        let mut rem = data.len();

        while rem > 0 {
            let dst = packetizer.spare_capacity_mut();
            let to_write = rem.min(dst.len());

            dst[..to_write].write_copy_of_slice(&data[len - rem..len - rem + to_write]);

            rem -= to_write;
            unsafe {
                packetizer.bytes_written(to_write);
            }

            while let Some(msg) = packetizer.next_message() {
                msgs.push(msg);
            }
        }
    }
});
