use bytes::Bytes;

use crate::{common::ByteStr, ext::FmtExt};

#[derive(Debug)]
pub struct Headers {
    headers: Vec<Header>,
}

impl Headers {
    pub(crate) fn from_buffer(buffer: Vec<Header>) -> Headers {
        Self { headers: buffer }
    }

    pub fn get(&self, key: &str) -> Option<&Header> {
        self.headers.iter().find(|e| e.name == key)
    }
}

pub struct Header {
    name: ByteStr,
    value: Bytes,
    is_str: bool,
}

impl Header {
    pub(crate) fn new(name: ByteStr, value: Bytes) -> Self {
        Self { name, value, is_str: false }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        match self.is_str {
            false => std::str::from_utf8(&self.value),
            // SAFETY: string validity is tracked and is immutable
            true => Ok(unsafe { std::str::from_utf8_unchecked(&self.value) }),
        }
    }

    pub fn to_str(&mut self) -> Result<&str, std::str::Utf8Error> {
        if !self.is_str {
            std::str::from_utf8(&self.value)?;
            self.is_str = true;
        }
        self.as_str()
    }

    /// Returns interator that parse `"; "` sequence.
    pub fn as_sequence(&self) -> Sequence {
        Sequence {
            value: self.as_str().ok().map(|e| e.split("; ")),
        }
    }
}

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

impl std::fmt::Debug for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Header")
            .field("name", &self.name)
            .field("value", &self.value.lossy())
            .finish()
    }
}
