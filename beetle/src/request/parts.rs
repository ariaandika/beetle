use crate::{
    common::ByteStr,
    headers::HeaderMap,
    http::{Extensions, Method, Version},
};

/// HTTP Request Parts.
#[derive(Default)]
pub struct Parts {
    method: Method,
    path: ByteStr,
    version: Version,
    headers: HeaderMap,
    extensions: Extensions,
}

impl Parts {
    pub(crate) fn new(
        method: Method,
        path: ByteStr,
        version: Version,
        headers: HeaderMap,
        extensions: Extensions,
    ) -> Self {
        Self {
            method,
            path,
            version,
            headers,
            extensions,
        }
    }

    /// Returns HTTP Method.
    pub fn method(&self) -> Method {
        self.method
    }

    /// Returns HTTP Path.
    pub fn path(&self) -> &ByteStr {
        &self.path
    }

    /// Returns HTTP Version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Returns HTTP Headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

impl std::fmt::Debug for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Parts")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("version", &self.version)
            .field("headers", &self.headers())
            .finish()
    }
}

