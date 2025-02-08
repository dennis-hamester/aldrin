use std::borrow::Borrow;
use std::ops::Deref;

/// Wrapper for `Vec<u8>` to enable `Serialize` and `Deserialize` specializations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "fuzzing", derive(arbitrary::Arbitrary))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    pub fn new<T: Into<Vec<u8>>>(bytes: T) -> Self {
        Self(bytes.into())
    }
}

impl Deref for Bytes {
    type Target = ByteSlice;

    fn deref(&self) -> &ByteSlice {
        ByteSlice::new(&self.0)
    }
}

impl AsRef<ByteSlice> for Bytes {
    fn as_ref(&self) -> &ByteSlice {
        self
    }
}

impl Borrow<ByteSlice> for Bytes {
    fn borrow(&self) -> &ByteSlice {
        self
    }
}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(bytes: Bytes) -> Self {
        bytes.0
    }
}

impl PartialEq<ByteSlice> for Bytes {
    fn eq(&self, other: &ByteSlice) -> bool {
        **self == *other
    }
}

impl PartialEq<[u8]> for Bytes {
    fn eq(&self, other: &[u8]) -> bool {
        **self == *other
    }
}

impl PartialEq<Bytes> for [u8] {
    fn eq(&self, other: &Bytes) -> bool {
        *self == ***other
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for Bytes {
//     fn layout() -> Layout {
//         BuiltInType::Bytes.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::BYTES
//     }

//     fn add_references(_references: &mut References) {}
// }

/// Wrapper for `[u8]` to enable `Serialize` and `Deserialize` specializations.
#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct ByteSlice(pub [u8]);

impl ByteSlice {
    pub fn new<T: AsRef<[u8]> + ?Sized>(bytes: &T) -> &Self {
        let self_ptr = bytes.as_ref() as *const [u8] as *const Self;
        // Safe because of repr(transparent).
        unsafe { &*self_ptr }
    }
}

impl Deref for ByteSlice {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for ByteSlice {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl AsRef<ByteSlice> for [u8] {
    fn as_ref(&self) -> &ByteSlice {
        ByteSlice::new(self)
    }
}

impl<'a, T: AsRef<[u8]>> From<&'a T> for &'a ByteSlice {
    fn from(bytes: &'a T) -> Self {
        ByteSlice::new(bytes)
    }
}

impl ToOwned for ByteSlice {
    type Owned = Bytes;

    fn to_owned(&self) -> Bytes {
        Bytes::new(&self.0)
    }
}

impl PartialEq<Bytes> for ByteSlice {
    fn eq(&self, other: &Bytes) -> bool {
        *self == **other
    }
}

impl PartialEq<[u8]> for ByteSlice {
    fn eq(&self, other: &[u8]) -> bool {
        **self == *other
    }
}

impl PartialEq<ByteSlice> for [u8] {
    fn eq(&self, other: &ByteSlice) -> bool {
        *self == **other
    }
}

// #[cfg(feature = "introspection")]
// impl Introspectable for ByteSlice {
//     fn layout() -> Layout {
//         BuiltInType::Bytes.into()
//     }

//     fn lexical_id() -> LexicalId {
//         LexicalId::BYTES
//     }

//     fn add_references(_references: &mut References) {}
// }
