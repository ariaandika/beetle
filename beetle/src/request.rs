//! http request
use crate::{
    IntoResponse,
    common::ByteStr,
    http::{Extensions, Headers, Method, Version},
};

mod body;
mod parts;

pub mod futures;

pub use body::Body;
pub use parts::Parts;

/// A type that can be constructed from [`Request`].
///
/// This trait is used as request handler parameters.
pub trait FromRequest: Sized {
    type Error: IntoResponse;

    type Future: Future<Output = Result<Self, Self::Error>>;

    fn from_request(req: Request) -> Self::Future;
}

/// A type that can be constructed from request [`Parts`].
///
/// This trait is used as request handler parameters.
pub trait FromRequestParts: Sized {
    type Error: IntoResponse;

    type Future: Future<Output = Result<Self, Self::Error>>;

    fn from_request_parts(parts: &mut Parts) -> Self::Future;
}

/// HTTP Request.
pub struct Request {
    parts: Parts,
    body: Body,
}

/// Construction methods.
impl Request {
    /// Construct request from [`Parts`] and [`Body`].
    ///
    /// See also [`Request::into_parts`].
    pub fn from_parts(parts: Parts, body: Body) -> Request {
        Self { parts, body }
    }

    /// Destruct request into [`Parts`] and [`Body`].
    ///
    /// See also [`Request::from_parts`]
    pub fn into_parts(self) -> (Parts, Body) {
        (self.parts, self.body)
    }

    /// Comsume request into [`Body`].
    pub fn into_body(self) -> Body {
        self.body
    }

    pub fn extensions(&self) -> &Extensions {
        self.parts.extensions()
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        self.parts.extensions_mut()
    }
}

/// Delegate methods.
impl Request {
    /// Returns HTTP Method.
    pub fn method(&self) -> Method {
        self.parts.method()
    }

    /// Returns HTTP Path.
    pub fn path(&self) -> &ByteStr {
        self.parts.path()
    }

    /// Returns HTTP Version.
    pub fn version(&self) -> Version {
        self.parts.version()
    }

    /// Returns HTTP Headers.
    pub fn headers(&self) -> &Headers {
        self.parts.headers()
    }
}

impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("method", &self.parts.method())
            .field("path", &self.parts.path())
            .field("version", &self.parts.version())
            .field("headers", &self.parts.headers())
            .finish()
    }
}
