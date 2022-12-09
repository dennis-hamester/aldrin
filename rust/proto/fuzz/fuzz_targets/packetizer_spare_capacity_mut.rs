#![no_main]
#![feature(maybe_uninit_write_slice)]

use aldrin_proto::message::Packetizer;
use libfuzzer_sys::fuzz_target;
use std::mem::MaybeUninit;

fuzz_target!(|data: Vec<Vec<u8>>| {
    let mut packetizer = Packetizer::new();

    for data in data {
        let len = data.len();
        let mut rem = data.len();

        while rem > 0 {
            let dst = packetizer.spare_capacity_mut();
            let to_write = rem.min(dst.len());

            MaybeUninit::write_slice(&mut dst[..to_write], &data[len - rem..len - rem + to_write]);
            rem -= to_write;
            unsafe {
                packetizer.bytes_written(to_write);
            }

            while let Some(_) = packetizer.next_message() {}
        }
    }
});
