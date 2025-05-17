use crate::{
    headers::HeaderMap,
    http::{Extensions, StatusCode, Version},
};

/// HTTP Response Parts.
#[derive(Default)]
pub struct Parts {
    version: Version,
    status: StatusCode,
    headers: HeaderMap,
    extensions: Extensions,
}

impl Parts {
    pub(crate) fn new(
        version: Version,
        status: StatusCode,
        headers: HeaderMap,
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
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns HTTP Headers.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
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

