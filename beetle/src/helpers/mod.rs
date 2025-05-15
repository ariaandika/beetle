//! helper types and traits

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "json")]
#[doc(inline)]
pub use json::Json;

/// service which holds another service
pub trait Layer<S> {
    type Service;
    fn layer(self, service: S) -> Self::Service;
}

/// represent two type that implement the same trait
pub enum Either<L,R> {
    Left(L),
    Right(R),
}

/// return bad request for error
///
/// implement [`IntoResponse`] with bad request and error message as body
#[derive(Debug)]
pub struct BadRequest<E>(pub E);

mod bad_request {
    use crate::IntoResponse;

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

