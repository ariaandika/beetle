use crate::http::{Extensions, Headers, StatusCode, Version};

/// HTTP Response Parts.
#[derive(Default)]
pub struct Parts {
    version: Version,
    status: StatusCode,
    headers: Headers,
    extensions: Extensions,
}

impl Parts {
    pub(crate) fn new(
        version: Version,
        status: StatusCode,
        headers: Headers,
        extensions: Extensions,
    ) -> Self {
        Self {
            version,
            status,
            headers,
            extensions,
        }
    }

    /// Returns HTTP Version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns HTTP Status Code.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns mutable reference of HTTP Status Code.
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    /// Returns HTTP Headers.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Insert header.
    pub fn insert_header(&mut self, key: &[u8], value: &[u8]) {
        todo!()
        // self.headers.put(key);
        // self.headers.put(&b": "[..]);
        // self.headers.put(value);
        // self.headers.put(&b"\r\n"[..]);
        // if self.header_len >= HEADER_SIZE {
        //     return;
        // }
        // self.headers[self.header_len] = header;
        // self.header_len += 1;
    }
}

impl std::fmt::Debug for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parts")
            .field("version", &self.version)
            .field("status", &self.status)
            .field("headers", &self.headers)
            .finish()
    }
}

