use std::convert::Infallible;

use super::{Branch, Matcher, State};
use crate::{
    helpers::Layer,
    request::Request,
    response::Response,
    service::{HttpService, Service, http::NotFound},
};

/// route builder
///
/// see [module level documentation](self) for more on routing
pub struct Router<S> {
    inner: S,
}

impl Router<NotFound> {
    /// create new `Router`
    pub fn new() -> Router<NotFound> {
        Router { inner: NotFound }
    }
}

impl<S> Router<S> {
    /// create new `Router` with custom fallback instead of 404 NotFound
    pub fn with_fallback(fallback: S) -> Router<S> {
        Router { inner: fallback }
    }

    /// layer current router service
    ///
    /// this is low level way to interact with `Router`
    ///
    /// see [`Layer`] for more information
    pub fn layer<L>(self, layer: L) -> Router<L::Service>
    where
        L: Layer<S>,
    {
        Router {
            inner: layer.layer(self.inner),
        }
    }

    /// assign new route
    pub fn route<R>(self, matcher: impl Into<Matcher>, route: R) -> Router<Branch<R, S>> {
        Router {
            inner: Branch::new(matcher, route, self.inner),
        }
    }

    pub fn state<T>(self, state: T) -> Router<State<T, S>> {
        Router {
            inner: State::new(state, self.inner),
        }
    }
}

impl<S> Service<Request> for Router<S>
where
    S: HttpService
{
    type Response = Response;
    type Error = Infallible;
    type Future = S::Future;

    fn call(&self, req: Request) -> Self::Future {
        self.inner.call(req)
    }
}

impl Default for Router<NotFound> {
    fn default() -> Self {
        Self::new()
    }
}

