
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

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

