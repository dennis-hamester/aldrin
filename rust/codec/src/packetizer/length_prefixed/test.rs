use super::{LengthPrefixed, Packetizer};

#[test]
fn decode_too_long() {
    let mut length_prefixed = LengthPrefixed::builder().max_length(8).build();

    // Decode a short packet that claims to be longer than 8 bytes.
    length_prefixed
        .decode(&mut [9, 0, 0, 0, 1][..].into())
        .unwrap_err();

    // Decode a packet that is actually too long.
    length_prefixed
        .decode(&mut [9, 0, 0, 0, 1, 2, 3, 4, 5][..].into())
        .unwrap_err();
}
