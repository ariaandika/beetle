use bytes::Bytes;
use std::{mem::take, str::from_utf8};

use crate::common::ByteStr;

/// HTTP Header Value.
pub struct HeaderValue {
    repr: Repr,
}

enum Repr {
    Bytes(Bytes),
    Str(ByteStr),
}

impl HeaderValue {
    pub(crate) const PLACEHOLDER: Self = Self {
        repr: Repr::Bytes(Bytes::new()),
    };

    /// Create new [`HeaderValue`] from bytes.
    pub fn new(value: impl Into<Bytes>) -> Self {
        Self {
            repr: Repr::Bytes(value.into()),
        }
    }

    /// Create new [`HeaderValue`] from string.
    pub fn new_str(value: impl Into<ByteStr>) -> Self {
        Self {
            repr: Repr::Str(value.into()),
        }
    }

    /// Try to parse value as [`str`].
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        match &self.repr {
            Repr::Bytes(b) => from_utf8(b),
            Repr::Str(s) => Ok(s),
        }
    }

    /// Try to parse value as [`str`] and cache the result.
    pub fn to_str(&mut self) -> Result<&str, std::str::Utf8Error> {
        match self.repr {
            Repr::Bytes(ref mut b) => {
                let s = ByteStr::from_utf8(take(b))?;
                self.repr = Repr::Str(s);
                self.as_str()
            }
            Repr::Str(ref s) => Ok(s.as_str()),
        }
    }

    /// Parse `"; "` separated value as [`Iterator`].
    pub fn as_sequence(&self) -> Sequence {
        Sequence {
            value: self.as_str().ok().map(|e| e.split("; ")),
        }
    }
}

impl std::fmt::Debug for HeaderValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeaderValue").finish()
    }
}

/// Parse `"; "` separated value as [`Iterator`].
///
/// This struct is returned from [`as_sequence`][HeaderValue::as_sequence].
#[derive(Debug)]
pub struct Sequence<'a> {
    value: Option<std::str::Split<'a,&'static str>>,
}

impl<'a> Iterator for Sequence<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.value.as_mut()?.next()
    }
}

