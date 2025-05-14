use std::convert::Infallible;

use super::Matcher;
use crate::{
    futures::{EitherInto, FutureExt},
    http::Method,
    request::Request,
    response::Response,
    routing::handler::HandlerService,
    service::{HttpService, Service, http::MethodNotAllowed},
};

/// service that match request and delegate to either service
///
/// user typically does not interact with this directly,
/// instead use [`route`] method, or [`get`] or [`post`] function
///
/// [`route`]: Router::route
pub struct Branch<S,F> {
    matcher: Matcher,
    inner: S,
    fallback: F,
}

macro_rules! fn_router {
    ($name:ident $method:ident $doc:literal) => {
        #[doc = $doc]
        pub fn $name<F,S>(f: F) -> Branch<HandlerService<F,S>,MethodNotAllowed> {
            Branch {
                matcher: Method::$method.into(),
                inner: HandlerService::new(f),
                fallback: MethodNotAllowed,
            }
        }
    };
    (self $name:ident $method:ident $doc:literal) => {
        #[doc = $doc]
        pub fn $name<S2,F2>(self, f: F2) -> Branch<HandlerService<F2, S2>, Branch<S, F>> {
            Branch {
                matcher: Method::$method.into(),
                inner: HandlerService::new(f),
                fallback: self,
            }
        }
    };
}

fn_router!(get GET "setup GET service");
fn_router!(post POST "setup POST service");
fn_router!(put PUT "setup PUT service");
fn_router!(patch PATCH "setup PATCH service");
fn_router!(delete DELETE "setup DELETE service");

impl<S, F> Branch<S, F> {
    pub fn new(matcher: impl Into<Matcher>, inner: S, fallback: F) -> Self {
        Self { matcher: matcher.into(), inner, fallback }
    }

    fn_router!(self get GET "add GET service");
    fn_router!(self post POST "add POST service");
    fn_router!(self put PUT "add PUT service");
    fn_router!(self patch PATCH "add PATCH service");
    fn_router!(self delete DELETE "add DELETE service");
}

impl<S,F> Service<Request> for Branch<S,F>
where
    S: HttpService,
    F: HttpService,
{
    type Response = Response;
    type Error = Infallible;
    type Future = EitherInto<S::Future,F::Future,Result<Response,Infallible>>;

    fn call(&self, req: Request) -> Self::Future {
        match self.matcher == req {
            true => self.inner.call(req).left_into(),
            false => self.fallback.call(req).right_into(),
        }
    }
}

