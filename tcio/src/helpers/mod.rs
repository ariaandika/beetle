//! helper types and traits

#[cfg(feature = "json")]
pub mod json;

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

