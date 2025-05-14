use bytes::Bytes;
use futures_core::Stream;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use super::Body;

/// Futures returned from [`Body::collect`].
#[derive(Debug)]
#[must_use = "`Stream` does nothing unless polled"]
pub struct BodyStream {
    body: Body,
}

impl BodyStream {
    pub fn new(body: Body) -> Self {
        Self { body }
    }
}

impl Stream for BodyStream {
    type Item = Result<Bytes, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use super::BodyKind;
        use std::task::{Poll::Ready, ready};

        let body = match &mut self.body.kind {
            BodyKind::Bytes(b) => return Ready(Some(Ok(std::mem::take(b)))),
            BodyKind::Tcp(ok) => ok,
        };

        ready!(Pin::new(&mut *body).poll_read(cx)?);

        Ready(Some(Ok(std::mem::take(body.buffer_mut()).freeze())))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.body.content_len();
        (len,Some(len))
    }
}

