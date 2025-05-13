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
}

impl Header {
    pub(crate) fn new(name: ByteStr, value: Bytes) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &[u8] {
        &self.value
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
