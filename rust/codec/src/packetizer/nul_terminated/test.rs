use super::{NulTerminated, Packetizer};
use bytes::BytesMut;

#[test]
fn basic() {
    let mut nul_terminated = NulTerminated::new();
    let mut buf = BytesMut::new();

    nul_terminated
        .encode(b"Hello,"[..].into(), &mut buf)
        .unwrap();
    assert_eq!(buf, b"Hello,\0"[..]);

    nul_terminated
        .encode(b" World!"[..].into(), &mut buf)
        .unwrap();
    assert_eq!(buf, b"Hello,\0 World!\0"[..]);

    assert_eq!(
        nul_terminated.decode(&mut buf).unwrap().unwrap(),
        b"Hello,"[..]
    );
    assert_eq!(buf, b" World!\0"[..]);

    assert_eq!(
        nul_terminated.decode(&mut buf).unwrap().unwrap(),
        b" World!"[..]
    );
    assert_eq!(buf, b""[..]);

    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);
}

#[test]
fn initial_none() {
    let mut nul_terminated = NulTerminated::new();
    let mut buf = BytesMut::new();
    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);
}

#[test]
fn partial_decode() {
    let mut nul_terminated = NulTerminated::new();
    let mut buf = BytesMut::new();

    buf.extend_from_slice(b"Hello,");
    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b" World!");
    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b"\0");
    assert_eq!(
        nul_terminated.decode(&mut buf).unwrap().unwrap(),
        b"Hello, World!"[..]
    );

    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b"Hello,");
    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b" World!");
    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);

    buf.extend_from_slice(b"\0");
    assert_eq!(
        nul_terminated.decode(&mut buf).unwrap().unwrap(),
        b"Hello, World!"[..]
    );

    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);
}

#[test]
fn exceed_max_length_encode() {
    let mut nul_terminated = NulTerminated::with_max_length(4);
    let mut buf = BytesMut::new();

    nul_terminated.encode(b"123"[..].into(), &mut buf).unwrap();
    assert!(nul_terminated.encode(b"1234"[..].into(), &mut buf).is_err());
    assert_eq!(buf, b"123\0"[..]);

    nul_terminated.encode(b"456"[..].into(), &mut buf).unwrap();
    assert_eq!(buf, b"123\x00456\x00"[..]);
}

#[test]
fn exceed_max_length_decode() {
    let mut nul_terminated = NulTerminated::with_max_length(4);
    let mut buf = BytesMut::new();

    nul_terminated.encode(b"123"[..].into(), &mut buf).unwrap();
    assert_eq!(
        nul_terminated.decode(&mut buf).unwrap().unwrap(),
        b"123"[..]
    );

    assert!(buf.is_empty());
    buf.extend_from_slice(b"1234\0");
    assert!(nul_terminated.decode(&mut buf).is_err());

    buf.clear();
    buf.extend_from_slice(b"123");
    assert_eq!(nul_terminated.decode(&mut buf).unwrap(), None);
    buf.extend_from_slice(b"4");
    assert!(nul_terminated.decode(&mut buf).is_err());
}
