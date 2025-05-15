use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fmt, io,
    marker::PhantomData,
    pin::Pin,
    task::{
        Context,
        Poll::{self, Ready},
        ready,
    },
};

use crate::{
    FromRequest, IntoResponse, Request, Response, helpers::BadRequest, http::StatusCode, response,
};

pub struct Json<T>(pub T);

impl<T: DeserializeOwned> FromRequest for Json<T> {
    type Error = JsonFutureError;

    type Future = JsonFuture<T>;

    fn from_request(req: Request) -> Self::Future {
        JsonFuture {
            phase: Phase::P1 { req: Some(req) },
            _p: PhantomData,
        }
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(ok) => response::Body::bytes(ok).into_response(),
            Err(_err) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct JsonFuture<T> {
        #[pin]
        phase: Phase,
        _p: PhantomData<T>,
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    #[project = PhaseProject]
    pub enum Phase {
        P1 { req: Option<Request> },
        P2 { #[pin] f: <Bytes as FromRequest>::Future },
    }
}

impl<T: DeserializeOwned> Future for JsonFuture<T> {
    type Output = Result<Json<T>, JsonFutureError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut me = self.as_mut().project();

        loop {
            match me.phase.as_mut().project() {
                PhaseProject::P1 { req } => {
                    let Some("application/json") = req
                        .as_ref()
                        .unwrap()
                        .headers()
                        .get("content-type")
                        .and_then(|e| e.as_sequence().next())
                    else {
                        return Ready(Err(JsonFutureError::ContentType));
                    };

                    let f = Bytes::from_request(req.take().unwrap());
                    *me.phase = Phase::P2 { f };
                }
                PhaseProject::P2 { f } => {
                    let buffer = ready!(f.poll(cx)?);
                    return Ready(match serde_json::from_slice(&buffer) {
                        Ok(ok) => Ok(Json(ok)),
                        Err(err) => Err(err.into()),
                    });
                }
            }
        }
    }
}

pub enum JsonFutureError {
    /// `Content-Type` header is not `application/json`
    ContentType,
    Io(io::Error),
    Serde(serde_json::Error),
}

impl From<BadRequest<io::Error>> for JsonFutureError {
    fn from(value: BadRequest<io::Error>) -> Self {
        Self::Io(value.0)
    }
}

impl From<serde_json::Error> for JsonFutureError {
    fn from(value: serde_json::Error) -> Self {
        Self::Serde(value)
    }
}

impl std::error::Error for JsonFutureError {}

impl fmt::Display for JsonFutureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use JsonFutureError::*;
        match self {
            ContentType => f.write_str("`Content-Type` missmatch"),
            Io(e) => write!(f, "{e}"),
            Serde(error) => write!(f, "failed to parse json: {error}"),
        }
    }
}

impl fmt::Debug for JsonFutureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl IntoResponse for JsonFutureError {
    fn into_response(self) -> Response {
        BadRequest::new(self).into_response()
    }
}
