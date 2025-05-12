use std::{borrow::Cow, fmt, str::FromStr};

/// HTTP Method.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Method {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    CONNECT,
}

impl Method {
    /// Returns the string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            Method::HEAD => "HEAD",
            Method::CONNECT => "CONNECT",
        }
    }
}

impl FromStr for Method {
    type Err = UnknownMethod;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" | "get" => Ok(Self::GET),
            "POST" | "post" => Ok(Self::POST),
            "PUT" | "put" => Ok(Self::PUT),
            "PATCH" | "patch" => Ok(Self::PATCH),
            "DELETE" | "delete" => Ok(Self::DELETE),
            "HEAD" | "head" => Ok(Self::HEAD),
            "CONNECT" | "connect" => Ok(Self::CONNECT),
            _ => Err(UnknownMethod(Cow::Owned(s.into()))),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ===== Error =====

/// Error when parsing [`Method`].
pub struct UnknownMethod(Cow<'static, str>);

impl std::error::Error for UnknownMethod {}

impl fmt::Display for UnknownMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown http method: {}", self.0)
    }
}

impl fmt::Debug for UnknownMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}
