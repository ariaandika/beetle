use bytes::Bytes;
use std::{
    io,
    task::{Context, Poll},
};

/// HTTP Response Body.
pub struct Body {
    kind: Kind,
}

enum Kind {
    Bytes(Bytes),
}

impl Body {
    /// Create empty [`Body`].
    pub fn empty() -> Self {
        Self {
            kind: Kind::Bytes(Bytes::new()),
        }
    }

    /// Create [`Body`] with given bytes.
    pub fn bytes(bytes: impl Into<Bytes>) -> Self {
        Self {
            kind: Kind::Bytes(bytes.into()),
        }
    }

    /// Returns the body length.
    pub fn content_len(&self) -> usize {
        match &self.kind {
            Kind::Bytes(b) => b.len(),
        }
    }

    /// Poll for data.
    pub(crate) fn poll_data(&mut self, _: &mut Context) -> Poll<io::Result<Bytes>> {
        match &mut self.kind {
            Kind::Bytes(b) => Poll::Ready(Ok(std::mem::take(b))),
        }
    }

    /// Returns `true` if stream is exhausted.
    pub fn is_end_stream(&self) -> bool {
        match &self.kind {
            Kind::Bytes(b) => b.is_empty(),
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}
