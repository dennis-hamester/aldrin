use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion {
    major: u32,
    minor: u32,
}

impl ProtocolVersion {
    pub const V1_14: Self = Self::new(1, 14);
    pub const V1_15: Self = Self::new(1, 15);
    pub const V1_16: Self = Self::new(1, 16);
    pub const V1_17: Self = Self::new(1, 17);
    pub const V1_18: Self = Self::new(1, 18);
    pub const V1_19: Self = Self::new(1, 19);
    pub const V1_20: Self = Self::new(1, 20);

    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    pub const fn major(self) -> u32 {
        self.major
    }

    pub const fn minor(self) -> u32 {
        self.minor
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major(), self.minor())
    }
}

impl FromStr for ProtocolVersion {
    type Err = ProtocolVersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major, minor) = s.split_once('.').ok_or(ProtocolVersionParseError)?;

        let major = major.parse().map_err(|_| ProtocolVersionParseError)?;
        let minor = minor.parse().map_err(|_| ProtocolVersionParseError)?;

        Ok(Self::new(major, minor))
    }
}

#[derive(Error, Debug, Copy, Clone, PartialEq, Eq)]
#[error("failed to parse protocol version")]
pub struct ProtocolVersionParseError;

#[cfg(test)]
mod test {
    use super::{ProtocolVersion, ProtocolVersionParseError};

    #[test]
    fn parse_protocol_version() {
        assert_eq!("1.14".parse(), Ok(ProtocolVersion::V1_14));
        assert_eq!("1.15".parse(), Ok(ProtocolVersion::V1_15));
        assert_eq!("1.16".parse(), Ok(ProtocolVersion::V1_16));
        assert_eq!("1.17".parse(), Ok(ProtocolVersion::V1_17));
        assert_eq!("1.18".parse(), Ok(ProtocolVersion::V1_18));
        assert_eq!("1.19".parse(), Ok(ProtocolVersion::V1_19));
        assert_eq!("1.20".parse(), Ok(ProtocolVersion::V1_20));

        assert_eq!(
            "1.4294967296".parse::<ProtocolVersion>(),
            Err(ProtocolVersionParseError)
        );

        assert_eq!(
            "1.".parse::<ProtocolVersion>(),
            Err(ProtocolVersionParseError)
        );

        assert_eq!(
            ".14".parse::<ProtocolVersion>(),
            Err(ProtocolVersionParseError)
        );

        assert_eq!(
            "".parse::<ProtocolVersion>(),
            Err(ProtocolVersionParseError)
        );
    }
}
