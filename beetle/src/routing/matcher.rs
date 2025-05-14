use crate::{http::Method, request::Request};

/// partially match request
#[derive(Clone, Default)]
pub struct Matcher {
    path: Option<&'static str>,
    method: Option<Method>,
}

impl PartialEq<Request> for Matcher {
    fn eq(&self, other: &Request) -> bool {
        if let Some(path) = self.path {
            if other.path().eq(path) {
                return false;
            }
        }
        if let Some(method) = &self.method {
            if method != &other.method() {
                return false;
            }
        }
        true
    }
}

macro_rules! matcher_from {
    ($id:pat,$ty:ty => $($tt:tt)*) => {
        impl From<$ty> for Matcher {
            fn from($id: $ty) -> Self {
                Self $($tt)*
            }
        }
    };
}

matcher_from!(_,() => ::default());
matcher_from!(value,Method => { method: Some(value), ..Default::default() });
matcher_from!(value,&'static str => { path: Some(value), ..Default::default() });
matcher_from!((p,m),(&'static str,Method) => { path: Some(p), method: Some(m) });
