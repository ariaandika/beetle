use bytes::Bytes;

use super::HeaderParser;
use crate::{
    common::ByteStr,
    ext::FmtExt,
    http::{Extensions, Method, Version},
};

/// an http request parts
pub struct Parts {
    method: Method,
    path: ByteStr,
    version: Version,
    headers: Bytes,
    extensions: Extensions,
}

impl Parts {
    pub(crate) fn new(
        method: Method,
        path: ByteStr,
        version: Version,
        headers: Bytes,
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
    pub fn headers(&self) -> Headers {
        Headers { headers: &self.headers }
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

pub struct Headers<'a> {
    headers: &'a [u8],
}

impl Headers<'_> {
    pub fn get(&self, k: &str) -> Option<&[u8]> {
        HeaderParser::new(self.headers)
            .find(|e| e.0 == k.as_bytes())
            .map(|e| e.1)
    }
}

impl std::fmt::Debug for Headers<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m = f.debug_map();

        for (k, v) in HeaderParser::new(self.headers) {
            m.key(&k.lossy()).value(&v.lossy());
        }

        m.finish()
    }
}

