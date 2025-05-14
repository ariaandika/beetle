use bytes::Bytes;
use std::{
    io,
    pin::Pin,
    task::{
        Context,
        Poll::{self, Ready},
        ready,
    },
};

use super::{Body, BodyKind};

/// Futures returned from [`Body::collect`].
#[derive(Debug)]
#[must_use = "`Future` does nothing unless polled or .awaited"]
pub struct Collect {
    body: Body,
}

impl Collect {
    pub fn new(body: Body) -> Self {
        Self { body }
    }
}

impl Future for Collect {
    type Output = io::Result<Bytes>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let body = match &mut self.body.kind {
            BodyKind::Bytes(b) => return Ready(Ok(std::mem::take(b))),
            BodyKind::Tcp(ok) => ok,
        };

        while body.remaining() != 0 {
            ready!(Pin::new(&mut *body).poll_read(cx)?);
        }

        Ready(Ok(std::mem::take(body.buffer_mut()).freeze()))
    }
}
