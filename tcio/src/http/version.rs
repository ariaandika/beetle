use std::{borrow::Cow, fmt, str::FromStr};

/// HTTP Version.
#[derive(Clone, Copy, Default, Debug)]
pub enum Version {
    V10,
    #[default]
    V11,
    V2,
}

impl Version {
    /// Returns string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Version::V10 => "HTTP/1.0",
            Version::V11 => "HTTP/1.1",
            Version::V2 =>  "HTTP/2",
        }
    }
}

impl FromStr for Version {
    type Err = UnknownVersion;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(Version::V10),
            "HTTP/1.1" => Ok(Version::V11),
            "HTTP/2" => Ok(Version::V2),
            _ => Err(UnknownVersion(Cow::Owned(s.into()))),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ===== Error =====

/// Error when parsing [`Method`].
pub struct UnknownVersion(Cow<'static, str>);

impl std::error::Error for UnknownVersion {}

impl fmt::Display for UnknownVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown http version: {}", self.0)
    }
}

impl fmt::Debug for UnknownVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

