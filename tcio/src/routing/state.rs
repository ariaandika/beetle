use crate::{
    request::Request,
    service::{HttpService, Service},
};

pub struct State<T, S> {
    #[allow(dead_code)]
    state: T,
    inner: S,
}

impl<T, S> Service<Request> for State<T, S>
where
    T: Clone + Send + Sync + 'static,
    S: HttpService,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&self, req: Request) -> Self::Future {
        // TODO: req.extensions_mut().insert(self.state.clone());
        self.inner.call(req)
    }
}
