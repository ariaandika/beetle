//! http response
use bytes::{BufMut, BytesMut};

use crate::body::Body;
use crate::http::{StatusCode, Version};

mod into_response;
mod writer;

pub use writer::{check, write};

/// an http response parts
#[derive(Default)]
pub struct Parts {
    version: Version,
    status: StatusCode,
    headers: BytesMut,
    // header_len: usize,
}

impl Parts {
    /// getter for http version
    pub fn version(&self) -> Version {
        self.version
    }

    /// getter for http status
    pub fn status(&self) -> StatusCode {
        self.status
    }

    // /// getter for http headers
    // pub fn headers(&self) -> &[Header] {
    //     &self.headers[..self.header_len]
    // }

    /// insert new header
    pub fn insert_header(&mut self, key: &[u8], value: &[u8]) {
        self.headers.put(key);
        self.headers.put(&b": "[..]);
        self.headers.put(value);
        self.headers.put(&b"\r\n"[..]);
        // if self.header_len >= HEADER_SIZE {
        //     return;
        // }
        // self.headers[self.header_len] = header;
        // self.header_len += 1;
    }
}

/// an http response
pub struct Response {
    parts: Parts,
    body: Body,
}

/// construction methods
impl Response {
    pub(crate) fn empty() -> Self {
        Self {
            parts: <_>::default(),
            body: Body::empty(),
        }
    }

    /// construct new response with body
    pub fn new(body: Body) -> Self {
        Self {
            parts: <_>::default(),
            body,
        }
    }

    /// construct response from parts
    ///
    /// see also [`Response::into_parts`]
    pub fn from_parts(parts: Parts, body: Body) -> Response {
        Response { parts, body }
    }

    /// destruct response into parts
    ///
    /// see also [`Response::from_parts`]
    pub fn into_parts(self) -> (Parts, Body) {
        (self.parts,self.body)
    }

    /// destruct response into [`ResBody`]
    pub fn into_body(self) -> Body {
        self.body
    }
}

/// delegate methods
impl Response {
    /// getter for http version
    pub fn version(&self) -> Version {
        self.parts.version
    }

    /// getter for http status
    pub fn status(&self) -> StatusCode {
        self.parts.status
    }

    // /// getter for http headers
    // pub fn headers(&self) -> &[Header] {
    //     self.parts.headers()
    // }
}

impl Default for Response {
    fn default() -> Self {
        Self::empty()
    }
}

/// a type that can be converted into response
///
/// this trait is used as request handler return type
pub trait IntoResponse {
    fn into_response(self) -> Response;
}

/// a type that can be converted into response parts
///
/// this trait is used as request handler return type
pub trait IntoResponseParts {
    fn into_response_parts(self, parts: Parts) -> Parts;
}

impl std::fmt::Debug for Parts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parts")
            .field("version", &self.version)
            .field("status", &self.status)
            // .field("headers", &self.headers())
            .finish()
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("version", &self.parts.version)
            .field("status", &self.parts.status)
            // .field("headers", &self.parts.headers())
            .finish()
    }
}


/// return bad request for error
///
/// implement [`IntoResponse`] with bad request and error message as body
#[derive(Debug)]
pub struct BadRequest<E>(pub E);

mod bad_request {
    use super::*;

    impl<E> BadRequest<E> {
        /// create new [`BadRequest`]
        pub fn new(inner: E) -> Self {
            Self(inner)
        }

        pub fn map<T: From<E>>(self) -> BadRequest<T> {
            BadRequest(self.0.into())
        }
    }

    impl<E> From<E> for BadRequest<E>
    where
        E: std::error::Error,
    {
        fn from(value: E) -> Self {
            Self(value)
        }
    }

    impl<E> IntoResponse for BadRequest<E>
    where
        E: std::fmt::Display
    {
        fn into_response(self) -> crate::Response {
            (crate::http::StatusCode::BAD_REQUEST, self.0.to_string()).into_response()
        }
    }

    impl<E> std::error::Error for BadRequest<E> where E: std::error::Error { }

    impl<E> std::fmt::Display for BadRequest<E> where E: std::fmt::Display {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            E::fmt(&self.0, f)
        }
    }
}

