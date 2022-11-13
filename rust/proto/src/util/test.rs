use super::{BufExt, BufMutExt};
use crate::error::DeserializeError;
use crate::message::MessageKind;
use crate::value::ValueKind;
use bytes::BytesMut;

#[test]
fn try_put_value_kind() {
    let mut buf = BytesMut::new();
    buf.try_put_discriminant_u8(ValueKind::U32).unwrap();
    assert_eq!(*buf, [ValueKind::U32.into()]);

    let mut buf = BytesMut::new();
    buf.try_put_discriminant_u8(MessageKind::CallFunction)
        .unwrap();
    assert_eq!(*buf, [MessageKind::CallFunction.into()]);
}

#[test]
fn try_put_u8() {
    let mut buf = BytesMut::new();
    buf.try_put_u8(0).unwrap();
    assert_eq!(*buf, [0]);

    let mut buf = BytesMut::new();
    buf.try_put_u8(123).unwrap();
    assert_eq!(*buf, [123]);
}

#[test]
fn try_put_i8() {
    let mut buf = BytesMut::new();
    buf.try_put_i8(0).unwrap();
    assert_eq!(*buf, [0]);

    let mut buf = BytesMut::new();
    buf.try_put_i8(123).unwrap();
    assert_eq!(*buf, [123]);

    let mut buf = BytesMut::new();
    buf.try_put_i8(-123).unwrap();
    assert_eq!(*buf, [133]);
}

#[test]
fn try_put_u32_le() {
    let mut buf = BytesMut::new();
    buf.try_put_u32_le(0x12345678).unwrap();
    assert_eq!(*buf, [0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn try_put_u64_le() {
    let mut buf = BytesMut::new();
    buf.try_put_u64_le(0x123456789abcdef0).unwrap();
    assert_eq!(*buf, [0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn try_put_varint_u16_le() {
    let mut buf = BytesMut::new();
    buf.try_put_varint_u16_le(0x0000).unwrap();
    assert_eq!(*buf, [0x00]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u16_le(0x00fd).unwrap();
    assert_eq!(*buf, [0xfd]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u16_le(0x00fe).unwrap();
    assert_eq!(*buf, [254, 0xfe]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u16_le(0x00ff).unwrap();
    assert_eq!(*buf, [254, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u16_le(0x0100).unwrap();
    assert_eq!(*buf, [255, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u16_le(0xffff).unwrap();
    assert_eq!(*buf, [255, 0xff, 0xff]);
}

#[test]
fn try_put_varint_i16_le() {
    let mut buf = BytesMut::new();
    buf.try_put_varint_i16_le(0).unwrap();
    assert_eq!(*buf, [0]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i16_le(1).unwrap();
    assert_eq!(*buf, [2]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i16_le(i16::MAX).unwrap();
    assert_eq!(*buf, [255, 254, 255]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i16_le(i16::MIN).unwrap();
    assert_eq!(*buf, [255, 255, 255]);
}

#[test]
fn try_put_varint_u32_le() {
    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x00000000).unwrap();
    assert_eq!(*buf, [0x00]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x000000fb).unwrap();
    assert_eq!(*buf, [0xfb]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x000000fc).unwrap();
    assert_eq!(*buf, [252, 0xfc]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x000000ff).unwrap();
    assert_eq!(*buf, [252, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x00000100).unwrap();
    assert_eq!(*buf, [253, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x0000ffff).unwrap();
    assert_eq!(*buf, [253, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x00010000).unwrap();
    assert_eq!(*buf, [254, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x00ffffff).unwrap();
    assert_eq!(*buf, [254, 0xff, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0x01000000).unwrap();
    assert_eq!(*buf, [255, 0x00, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u32_le(0xffffffff).unwrap();
    assert_eq!(*buf, [255, 0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn try_put_varint_i32_le() {
    let mut buf = BytesMut::new();
    buf.try_put_varint_i32_le(0).unwrap();
    assert_eq!(*buf, [0]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i32_le(1).unwrap();
    assert_eq!(*buf, [2]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i32_le(-1).unwrap();
    assert_eq!(*buf, [1]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i32_le(i32::MAX).unwrap();
    assert_eq!(*buf, [255, 254, 255, 255, 255]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i32_le(i32::MIN).unwrap();
    assert_eq!(*buf, [255, 255, 255, 255, 255]);
}

#[test]
fn try_put_varint_u64_le() {
    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000000000000000).unwrap();
    assert_eq!(*buf, [0x00]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x00000000000000f7).unwrap();
    assert_eq!(*buf, [0xf7]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x00000000000000f8).unwrap();
    assert_eq!(*buf, [248, 0xf8]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x00000000000000ff).unwrap();
    assert_eq!(*buf, [248, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000000000000100).unwrap();
    assert_eq!(*buf, [249, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x000000000000ffff).unwrap();
    assert_eq!(*buf, [249, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000000000010000).unwrap();
    assert_eq!(*buf, [250, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000000000ffffff).unwrap();
    assert_eq!(*buf, [250, 0xff, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000000001000000).unwrap();
    assert_eq!(*buf, [251, 0x00, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x00000000ffffffff).unwrap();
    assert_eq!(*buf, [251, 0xff, 0xff, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000000100000000).unwrap();
    assert_eq!(*buf, [252, 0x00, 0x00, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x000000ffffffffff).unwrap();
    assert_eq!(*buf, [252, 0xff, 0xff, 0xff, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000010000000000).unwrap();
    assert_eq!(*buf, [253, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0000ffffffffffff).unwrap();
    assert_eq!(*buf, [253, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0001000000000000).unwrap();
    assert_eq!(*buf, [254, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x00ffffffffffffff).unwrap();
    assert_eq!(*buf, [254, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0x0100000000000000).unwrap();
    assert_eq!(*buf, [255, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_u64_le(0xffffffffffffffff).unwrap();
    assert_eq!(*buf, [255, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn try_put_varint_i64_le() {
    let mut buf = BytesMut::new();
    buf.try_put_varint_i64_le(0).unwrap();
    assert_eq!(*buf, [0]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i64_le(1).unwrap();
    assert_eq!(*buf, [2]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i64_le(-1).unwrap();
    assert_eq!(*buf, [1]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i64_le(i64::MAX).unwrap();
    assert_eq!(*buf, [255, 254, 255, 255, 255, 255, 255, 255, 255]);

    let mut buf = BytesMut::new();
    buf.try_put_varint_i64_le(i64::MIN).unwrap();
    assert_eq!(*buf, [255, 255, 255, 255, 255, 255, 255, 255, 255]);
}

#[test]
fn try_put_slice() {
    let mut buf = BytesMut::new();
    buf.try_put_slice([]).unwrap();
    assert_eq!(*buf, []);

    let mut buf = BytesMut::new();
    buf.try_put_slice([1, 2, 3]).unwrap();
    assert_eq!(*buf, [1, 2, 3]);
}

#[test]
fn try_get_discriminant_u8() {
    let mut buf = &[ValueKind::U32.into()][..];
    assert_eq!(buf.try_get_discriminant_u8(), Ok(ValueKind::U32));
    assert_eq!(*buf, []);

    let mut buf = &[MessageKind::CallFunction.into()][..];
    assert_eq!(buf.try_get_discriminant_u8(), Ok(MessageKind::CallFunction));
    assert_eq!(*buf, []);

    let mut buf = &[255][..];
    assert_eq!(
        buf.try_get_discriminant_u8::<ValueKind>(),
        Err(DeserializeError)
    );

    let buf = &[][..];
    assert_eq!(
        buf.try_peek_discriminant_u8::<ValueKind>(),
        Err(DeserializeError)
    );
}

#[test]
fn try_peek_discriminant_u8() {
    let buf = &[ValueKind::U32.into()][..];
    assert_eq!(buf.try_peek_discriminant_u8(), Ok(ValueKind::U32));

    let buf = &[MessageKind::CallFunction.into()][..];
    assert_eq!(
        buf.try_peek_discriminant_u8(),
        Ok(MessageKind::CallFunction)
    );

    let buf = &[255][..];
    assert_eq!(
        buf.try_peek_discriminant_u8::<ValueKind>(),
        Err(DeserializeError)
    );

    let buf = &[][..];
    assert_eq!(
        buf.try_peek_discriminant_u8::<ValueKind>(),
        Err(DeserializeError)
    );
}

#[test]
fn try_get_u8() {
    let mut buf = &[0][..];
    assert_eq!(buf.try_get_u8(), Ok(0));
    assert_eq!(*buf, []);

    let mut buf = &[255][..];
    assert_eq!(buf.try_get_u8(), Ok(255));
    assert_eq!(*buf, []);
}

#[test]
fn try_get_i8() {
    let mut buf = &[0][..];
    assert_eq!(buf.try_get_i8(), Ok(0));
    assert_eq!(*buf, []);

    let mut buf = &[255][..];
    assert_eq!(buf.try_get_i8(), Ok(-1));
    assert_eq!(*buf, []);

    let mut buf = &[127][..];
    assert_eq!(buf.try_get_i8(), Ok(127));
    assert_eq!(*buf, []);

    let mut buf = &[128][..];
    assert_eq!(buf.try_get_i8(), Ok(-128));
    assert_eq!(*buf, []);
}

#[test]
fn try_get_u32_le() {
    let mut buf = &[0x78, 0x56, 0x34, 0x12][..];
    assert_eq!(buf.try_get_u32_le(), Ok(0x12345678));
    assert_eq!(*buf, []);

    let mut buf = &[0, 0, 0][..];
    assert_eq!(buf.try_get_u32_le(), Err(DeserializeError));
}

#[test]
fn try_get_u64_le() {
    let mut buf = &[0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12][..];
    assert_eq!(buf.try_get_u64_le(), Ok(0x123456789abcdef0));
    assert_eq!(*buf, []);

    let mut buf = &[0, 0, 0, 0, 0, 0, 0][..];
    assert_eq!(buf.try_get_u64_le(), Err(DeserializeError));
}

#[test]
fn try_get_varint_u16_le() {
    let mut buf = &[0x00][..];
    assert_eq!(buf.try_get_varint_u16_le(), Ok(0x0000));
    assert_eq!(*buf, []);

    let mut buf = &[0xfd][..];
    assert_eq!(buf.try_get_varint_u16_le(), Ok(0x00fd));
    assert_eq!(*buf, []);

    let mut buf = &[254, 0xfe][..];
    assert_eq!(buf.try_get_varint_u16_le(), Ok(0x00fe));
    assert_eq!(*buf, []);

    let mut buf = &[254, 0xff][..];
    assert_eq!(buf.try_get_varint_u16_le(), Ok(0x00ff));
    assert_eq!(*buf, []);

    let mut buf = &[255, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u16_le(), Ok(0x0100));
    assert_eq!(*buf, []);

    let mut buf = &[255, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u16_le(), Ok(0xffff));
    assert_eq!(*buf, []);
}

#[test]
fn try_get_varint_i16_le() {
    let mut buf = &[0][..];
    assert_eq!(buf.try_get_varint_i16_le(), Ok(0));
    assert_eq!(*buf, []);

    let mut buf = &[2][..];
    assert_eq!(buf.try_get_varint_i16_le(), Ok(1));
    assert_eq!(*buf, []);

    let mut buf = &[255, 254, 255][..];
    assert_eq!(buf.try_get_varint_i16_le(), Ok(i16::MAX));
    assert_eq!(*buf, []);

    let mut buf = &[255, 255, 255][..];
    assert_eq!(buf.try_get_varint_i16_le(), Ok(i16::MIN));
    assert_eq!(*buf, []);
}

#[test]
fn try_get_varint_u32_le() {
    let mut buf = &[0x00][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x00000000));
    assert_eq!(*buf, []);

    let mut buf = &[0xfb][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x000000fb));
    assert_eq!(*buf, []);

    let mut buf = &[252, 0xfc][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x000000fc));
    assert_eq!(*buf, []);

    let mut buf = &[252, 0xff][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x000000ff));
    assert_eq!(*buf, []);

    let mut buf = &[253, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x00000100));
    assert_eq!(*buf, []);

    let mut buf = &[253, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x0000ffff));
    assert_eq!(*buf, []);

    let mut buf = &[254, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x00010000));
    assert_eq!(*buf, []);

    let mut buf = &[254, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x00ffffff));
    assert_eq!(*buf, []);

    let mut buf = &[255, 0x00, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0x01000000));
    assert_eq!(*buf, []);

    let mut buf = &[255, 0xff, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u32_le(), Ok(0xffffffff));
    assert_eq!(*buf, []);
}

#[test]
fn try_get_varint_i32_le() {
    let mut buf = &[0][..];
    assert_eq!(buf.try_get_varint_i32_le(), Ok(0));
    assert_eq!(*buf, []);

    let mut buf = &[2][..];
    assert_eq!(buf.try_get_varint_i32_le(), Ok(1));
    assert_eq!(*buf, []);

    let mut buf = &[1][..];
    assert_eq!(buf.try_get_varint_i32_le(), Ok(-1));
    assert_eq!(*buf, []);

    let mut buf = &[255, 254, 255, 255, 255][..];
    assert_eq!(buf.try_get_varint_i32_le(), Ok(i32::MAX));
    assert_eq!(*buf, []);

    let mut buf = &[255, 255, 255, 255, 255][..];
    assert_eq!(buf.try_get_varint_i32_le(), Ok(i32::MIN));
    assert_eq!(*buf, []);
}

#[test]
fn try_get_varint_u64_le() {
    let mut buf = &[0x00][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000000000000000));
    assert_eq!(*buf, []);

    let mut buf = &[0xf7][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x00000000000000f7));
    assert_eq!(*buf, []);

    let mut buf = &[248, 0xf8][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x00000000000000f8));
    assert_eq!(*buf, []);

    let mut buf = &[248, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x00000000000000ff));
    assert_eq!(*buf, []);

    let mut buf = &[249, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000000000000100));
    assert_eq!(*buf, []);

    let mut buf = &[249, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x000000000000ffff));
    assert_eq!(*buf, []);

    let mut buf = &[250, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000000000010000));
    assert_eq!(*buf, []);

    let mut buf = &[250, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000000000ffffff));
    assert_eq!(*buf, []);

    let mut buf = &[251, 0x00, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000000001000000));
    assert_eq!(*buf, []);

    let mut buf = &[251, 0xff, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x00000000ffffffff));
    assert_eq!(*buf, []);

    let mut buf = &[252, 0x00, 0x00, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000000100000000));
    assert_eq!(*buf, []);

    let mut buf = &[252, 0xff, 0xff, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x000000ffffffffff));
    assert_eq!(*buf, []);

    let mut buf = &[253, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000010000000000));
    assert_eq!(*buf, []);

    let mut buf = &[253, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0000ffffffffffff));
    assert_eq!(*buf, []);

    let mut buf = &[254, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0001000000000000));
    assert_eq!(*buf, []);

    let mut buf = &[254, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x00ffffffffffffff));
    assert_eq!(*buf, []);

    let mut buf = &[255, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0x0100000000000000));
    assert_eq!(*buf, []);

    let mut buf = &[255, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff][..];
    assert_eq!(buf.try_get_varint_u64_le(), Ok(0xffffffffffffffff));
    assert_eq!(*buf, []);
}

#[test]
fn try_get_varint_i64_le() {
    let mut buf = &[0][..];
    assert_eq!(buf.try_get_varint_i64_le(), Ok(0));
    assert_eq!(*buf, []);

    let mut buf = &[2][..];
    assert_eq!(buf.try_get_varint_i64_le(), Ok(1));
    assert_eq!(*buf, []);

    let mut buf = &[1][..];
    assert_eq!(buf.try_get_varint_i64_le(), Ok(-1));
    assert_eq!(*buf, []);

    let mut buf = &[255, 254, 255, 255, 255, 255, 255, 255, 255][..];
    assert_eq!(buf.try_get_varint_i64_le(), Ok(i64::MAX));
    assert_eq!(*buf, []);

    let mut buf = &[255, 255, 255, 255, 255, 255, 255, 255, 255][..];
    assert_eq!(buf.try_get_varint_i64_le(), Ok(i64::MIN));
    assert_eq!(*buf, []);
}

#[test]
fn try_copy_to_bytes() {
    let mut buf = &[1, 2, 3][..];
    assert_eq!(*buf.try_copy_to_bytes(3).unwrap(), [1, 2, 3]);
    assert_eq!(*buf, []);

    let mut buf = &[1, 2, 3][..];
    assert_eq!(*buf.try_copy_to_bytes(2).unwrap(), [1, 2]);
    assert_eq!(*buf, [3]);

    let mut buf = &[1, 2, 3][..];
    assert!(buf.try_copy_to_bytes(4).is_err());
}

#[test]
fn try_copy_to_slice() {
    let mut src = &[1, 2, 3][..];
    let mut dst = [0, 0, 0];
    src.try_copy_to_slice(&mut dst).unwrap();
    assert_eq!(dst, [1, 2, 3]);
    assert_eq!(*src, []);

    let mut src = &[1, 2, 3][..];
    let mut dst = [0, 0, 0];
    src.try_copy_to_slice(&mut dst[..2]).unwrap();
    assert_eq!(dst, [1, 2, 0]);
    assert_eq!(*src, [3]);

    let mut src = &[1, 2, 3][..];
    let mut dst = [0, 0, 0];
    src.try_copy_to_slice(&mut dst[..0]).unwrap();
    assert_eq!(dst, [0, 0, 0]);
    assert_eq!(*src, [1, 2, 3]);

    let mut src = &[1, 2, 3][..];
    let mut dst = [0, 0, 0, 0];
    assert!(src.try_copy_to_slice(&mut dst).is_err());
}

#[test]
fn try_skip() {
    let mut buf = &[1, 2][..];
    buf.try_skip(0).unwrap();
    assert_eq!(*buf, [1, 2]);

    let mut buf = &[1, 2][..];
    buf.try_skip(1).unwrap();
    assert_eq!(*buf, [2]);

    let mut buf = &[1, 2][..];
    buf.try_skip(2).unwrap();
    assert_eq!(*buf, []);

    let mut buf = &[1, 2][..];
    assert!(buf.try_skip(3).is_err());
}

#[test]
fn zigzag_encode_i16() {
    use super::zigzag_encode_i16;
    assert_eq!(zigzag_encode_i16(0), 0);
    assert_eq!(zigzag_encode_i16(1), 2);
    assert_eq!(zigzag_encode_i16(-1), 1);
    assert_eq!(zigzag_encode_i16(i16::MAX), u16::MAX - 1);
    assert_eq!(zigzag_encode_i16(i16::MIN), u16::MAX);
}

#[test]
fn zigzag_decode_i16() {
    use super::zigzag_decode_i16;
    assert_eq!(zigzag_decode_i16(0), 0);
    assert_eq!(zigzag_decode_i16(1), -1);
    assert_eq!(zigzag_decode_i16(2), 1);
    assert_eq!(zigzag_decode_i16(u16::MAX), i16::MIN);
    assert_eq!(zigzag_decode_i16(u16::MAX - 1), i16::MAX);
}

#[test]
fn zigzag_encode_i32() {
    use super::zigzag_encode_i32;
    assert_eq!(zigzag_encode_i32(0), 0);
    assert_eq!(zigzag_encode_i32(1), 2);
    assert_eq!(zigzag_encode_i32(-1), 1);
    assert_eq!(zigzag_encode_i32(i32::MAX), u32::MAX - 1);
    assert_eq!(zigzag_encode_i32(i32::MIN), u32::MAX);
}

#[test]
fn zigzag_decode_i32() {
    use super::zigzag_decode_i32;
    assert_eq!(zigzag_decode_i32(0), 0);
    assert_eq!(zigzag_decode_i32(1), -1);
    assert_eq!(zigzag_decode_i32(2), 1);
    assert_eq!(zigzag_decode_i32(u32::MAX), i32::MIN);
    assert_eq!(zigzag_decode_i32(u32::MAX - 1), i32::MAX);
}

#[test]
fn zigzag_encode_i64() {
    use super::zigzag_encode_i64;
    assert_eq!(zigzag_encode_i64(0), 0);
    assert_eq!(zigzag_encode_i64(1), 2);
    assert_eq!(zigzag_encode_i64(-1), 1);
    assert_eq!(zigzag_encode_i64(i64::MAX), u64::MAX - 1);
    assert_eq!(zigzag_encode_i64(i64::MIN), u64::MAX);
}

#[test]
fn zigzag_decode_i64() {
    use super::zigzag_decode_i64;
    assert_eq!(zigzag_decode_i64(0), 0);
    assert_eq!(zigzag_decode_i64(1), -1);
    assert_eq!(zigzag_decode_i64(2), 1);
    assert_eq!(zigzag_decode_i64(u64::MAX), i64::MIN);
    assert_eq!(zigzag_decode_i64(u64::MAX - 1), i64::MAX);
}
