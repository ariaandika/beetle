use crate::{
    common::ByteStr,
    http::{Extensions, Headers, Method, Version},
};

/// an http request parts
pub struct Parts {
    method: Method,
    path: ByteStr,
    version: Version,
    headers: Headers,
    extensions: Extensions,
}

impl Parts {
    pub(crate) fn new(
        method: Method,
        path: ByteStr,
        version: Version,
        headers: Headers,
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

    /// getter for http method
    pub fn method(&self) -> Method {
        self.method
    }

    /// getter for http path
    pub fn path(&self) -> &ByteStr {
        &self.path
    }

    /// getter for http version
    pub fn version(&self) -> Version {
        self.version
    }

    /// getter for http headers
    pub fn headers(&self) -> &Headers {
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

