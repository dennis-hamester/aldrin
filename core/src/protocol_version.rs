use crate::error::ProtocolVersionErrorKind;
use crate::ProtocolVersionError;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion {
    minor: Minor,
}

impl ProtocolVersion {
    pub const MAJOR: u32 = 1;
    pub const V1_14: Self = Self { minor: Minor::V14 };
    pub const V1_15: Self = Self { minor: Minor::V15 };
    pub const V1_16: Self = Self { minor: Minor::V16 };
    pub const V1_17: Self = Self { minor: Minor::V17 };
    pub const V1_18: Self = Self { minor: Minor::V18 };
    pub const V1_19: Self = Self { minor: Minor::V19 };
    pub const MIN: Self = Self::V1_14;
    pub const MAX: Self = Self::V1_19;

    pub const fn new(major: u32, minor: u32) -> Result<Self, ProtocolVersionError> {
        if major != Self::MAJOR {
            return Err(ProtocolVersionError {
                kind: ProtocolVersionErrorKind::InvalidMajor,
            });
        }

        match minor {
            14 => Ok(Self { minor: Minor::V14 }),
            15 => Ok(Self { minor: Minor::V15 }),
            16 => Ok(Self { minor: Minor::V16 }),
            17 => Ok(Self { minor: Minor::V17 }),
            18 => Ok(Self { minor: Minor::V18 }),
            19 => Ok(Self { minor: Minor::V19 }),

            _ => Err(ProtocolVersionError {
                kind: ProtocolVersionErrorKind::InvalidMinor,
            }),
        }
    }

    pub const fn major(&self) -> u32 {
        Self::MAJOR
    }

    pub const fn minor(&self) -> u32 {
        self.minor as u32
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Minor {
    V14 = 14,
    V15 = 15,
    V16 = 16,
    V17 = 17,
    V18 = 18,
    V19 = 19,
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major(), self.minor())
    }
}

impl FromStr for ProtocolVersion {
    type Err = ProtocolVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major, minor) = s.split_once('.').ok_or(ProtocolVersionErrorKind::Parse)?;
        let major = major.parse().map_err(|_| ProtocolVersionErrorKind::Parse)?;
        let minor = minor.parse().map_err(|_| ProtocolVersionErrorKind::Parse)?;
        Self::new(major, minor)
    }
}

#[cfg(test)]
mod test {
    use super::ProtocolVersion;
    use crate::error::ProtocolVersionErrorKind;

    #[test]
    fn parse_protocol_version() {
        assert_eq!("1.14".parse(), Ok(ProtocolVersion::V1_14));
        assert_eq!("1.15".parse(), Ok(ProtocolVersion::V1_15));
        assert_eq!("1.16".parse(), Ok(ProtocolVersion::V1_16));
        assert_eq!("1.17".parse(), Ok(ProtocolVersion::V1_17));
        assert_eq!("1.18".parse(), Ok(ProtocolVersion::V1_18));
        assert_eq!("1.19".parse(), Ok(ProtocolVersion::V1_19));

        assert_eq!(
            "1.13".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::InvalidMinor.into())
        );
        assert_eq!(
            "1.20".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::InvalidMinor.into())
        );

        assert_eq!(
            "0.14".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::InvalidMajor.into())
        );
        assert_eq!(
            "1.0".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::InvalidMinor.into())
        );
        assert_eq!(
            "1.4294967295".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::InvalidMinor.into())
        );

        assert_eq!(
            "1.".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::Parse.into())
        );
        assert_eq!(
            ".14".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::Parse.into())
        );
        assert_eq!(
            "".parse::<ProtocolVersion>(),
            Err(ProtocolVersionErrorKind::Parse.into())
        );
    }
}
