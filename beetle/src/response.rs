//! HTTP Response.
use crate::{
    headers::Headers,
    http::{StatusCode, Version},
};

mod body;
mod parts;

mod into_response;
mod writer;

pub use body::Body;
pub use parts::Parts;
pub use writer::{validate, write};

/// A type that can be converted into [`Response`].
///
/// This trait is used as request handler return type.
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

/// A type that can be converted into response [`Parts`].
///
/// This trait is used as request handler return type.
pub trait IntoResponseParts {
    fn into_response_parts(self, parts: Parts) -> Parts;
}

/// HTTP Response.
#[derive(Default)]
pub struct Response {
    parts: Parts,
    body: Body,
}

/// Construction methods.
impl Response {
    /// Construct new response with body.
    pub fn new(body: Body) -> Self {
        Self {
            parts: <_>::default(),
            body,
        }
    }

    /// Construct response from [`Parts`] and [`Body`].
    ///
    /// See also [`Response::into_parts`]
    pub fn from_parts(parts: Parts, body: Body) -> Response {
        Response { parts, body }
    }

    /// Destruct response into [`Parts`] and [`Body`].
    ///
    /// See also [`Response::from_parts`]
    pub fn into_parts(self) -> (Parts, Body) {
        (self.parts,self.body)
    }

    /// Consume response into [`Body`].
    pub fn into_body(self) -> Body {
        self.body
    }
}

/// Delegate methods.
impl Response {
    /// Returns HTTP Version.
    pub fn version(&self) -> Version {
        self.parts.version()
    }

    /// Returns HTTP Status Code.
    pub fn status(&self) -> StatusCode {
        self.parts.status()
    }

    /// Returns HTTP Headers.
    pub fn headers(&self) -> &Headers {
        self.parts.headers()
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("version", &self.parts.version())
            .field("status", &self.parts.status())
            .field("headers", &self.parts.headers())
            .finish()
    }
}

