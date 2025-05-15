//! asynchronous service
use std::convert::Infallible;

use crate::{request::Request, response::Response};

pub mod servicefn;
pub mod http;
pub mod tcp;

pub trait Service<Request> {
    type Response;

    type Error;

    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn call(&self, request: Request) -> Self::Future;
}

// ===== Http Service =====

/// A service that accept http request and return http response.
pub trait HttpService:
    Service<Request, Response = Response, Error = Infallible, Future = Self::HttpFuture>
    + Send
    + Sync
    + 'static
{
    type HttpFuture: Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + 'static;
}

impl<S> HttpService for S
where
    S: Service<Request, Response = Response, Error = Infallible> + Send + Sync + 'static,
    S::Future: Send + Sync + 'static,
{
    type HttpFuture = Self::Future;
}

// ===== Foreign type Service =====

impl<S, Req> Service<Req> for Box<S>
where
    S: Service<Req>,
{
    type Response = <S as Service<Req>>::Response;
    type Error = <S as Service<Req>>::Error;
    type Future = <S as Service<Req>>::Future;

    fn call(&self, request: Req) -> Self::Future {
        S::call(self, request)
    }
}

impl<S, Req> Service<Req> for std::sync::Arc<S>
where
    S: Service<Req>,
{
    type Response = <S as Service<Req>>::Response;
    type Error = <S as Service<Req>>::Error;
    type Future = <S as Service<Req>>::Future;

    fn call(&self, request: Req) -> Self::Future {
        S::call(self, request)
    }
}

