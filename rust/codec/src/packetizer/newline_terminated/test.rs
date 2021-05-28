use super::{NewlineTerminated, Packetizer};
use bytes::BytesMut;

#[test]
fn basic() {
    let mut newline_terminated = NewlineTerminated::new();
    let mut buf = BytesMut::new();

    newline_terminated
        .encode(b"Hello,"[..].into(), &mut buf)
        .unwrap();
    assert_eq!(buf, b"Hello,\n"[..]);

    newline_terminated
        .encode(b" World!"[..].into(), &mut buf)
        .unwrap();
    assert_eq!(buf, b"Hello,\n World!\n"[..]);

    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b"Hello,"[..]
    );
    assert_eq!(buf, b" World!\n"[..]);

    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b" World!"[..]
    );
    assert_eq!(buf, b""[..]);

    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);
}

#[test]
fn initial_none() {
    let mut newline_terminated = NewlineTerminated::new();
    let mut buf = BytesMut::new();
    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);
}

#[test]
fn partial_decode() {
    let mut newline_terminated = NewlineTerminated::new();
    let mut buf = BytesMut::new();

    buf.extend_from_slice(b"Hello,");
    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b" World!");
    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b"\n");
    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b"Hello, World!"[..]
    );

    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b"Hello,");
    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b" World!");
    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b"\n");
    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b"Hello, World!"[..]
    );

    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);
}

#[test]
fn exceed_max_length_encode() {
    let mut newline_terminated = NewlineTerminated::with_max_length(4);
    let mut buf = BytesMut::new();

    newline_terminated
        .encode(b"123"[..].into(), &mut buf)
        .unwrap();
    assert!(newline_terminated
        .encode(b"1234"[..].into(), &mut buf)
        .is_err());
    assert_eq!(buf, b"123\n"[..]);

    newline_terminated
        .encode(b"456"[..].into(), &mut buf)
        .unwrap();
    assert_eq!(buf, b"123\n456\n"[..]);
}

#[test]
fn exceed_max_length_decode() {
    let mut newline_terminated = NewlineTerminated::with_max_length(4);
    let mut buf = BytesMut::new();

    newline_terminated
        .encode(b"123"[..].into(), &mut buf)
        .unwrap();
    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b"123"[..]
    );

    assert!(buf.is_empty());
    buf.extend_from_slice(b"1234\n");
    assert!(newline_terminated.decode(&mut buf).is_err());

    buf.clear();
    buf.extend_from_slice(b"123");
    assert_eq!(newline_terminated.decode(&mut buf).unwrap(), None);
    buf.extend_from_slice(b"4");
    assert!(newline_terminated.decode(&mut buf).is_err());
}

#[test]
fn decode_with_cr() {
    let mut newline_terminated = NewlineTerminated::new();
    let mut buf = BytesMut::new();
    buf.extend_from_slice(b"123\r\n456\n789\r\n");

    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b"123"[..]
    );
    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b"456"[..]
    );
    assert_eq!(
        newline_terminated.decode(&mut buf).unwrap().unwrap(),
        b"789"[..]
    );
    assert!(buf.is_empty());
}
