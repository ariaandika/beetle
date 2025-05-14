//! Http Request and Response Body.
use bytes::Bytes;

mod tcp;
mod collect;
mod stream;

use tcp::TcpBody;

pub use collect::Collect;
pub use stream::BodyStream;

/// HTTP Request and Response Body.
///
/// To operate on [`Body`], caller can use either [`collect`][Self::collect] to buffer all request,
/// or [`stream`][Self::stream] to read bytes incrementally.
pub struct Body {
    kind: BodyKind,
}

enum BodyKind {
    Bytes(Bytes),
    #[allow(unused)]
    Tcp(TcpBody),
}

impl Body {
    pub(crate) fn empty() -> Self {
        Self {
            kind: BodyKind::Bytes(Bytes::new()),
        }
    }

    pub(crate) fn content_len(&self) -> usize {
        match &self.kind {
            BodyKind::Bytes(bytes) => bytes.len(),
            BodyKind::Tcp(body) => body.content_len(),
        }
    }

    pub(crate) fn bytes(bytes: impl Into<Bytes>) -> Self {
        Self {
            kind: BodyKind::Bytes(bytes.into()),
        }
    }

    #[cfg(feature = "tokio")]
    pub(crate) fn tcp(
        shared: std::sync::Arc<tokio::net::TcpStream>,
        content_len: usize,
        buffer: bytes::BytesMut,
    ) -> Self {
        Self {
            kind: BodyKind::Tcp(TcpBody::new(shared, content_len, buffer)),
        }
    }

    /// Read all body into [`BytesMut`].
    ///
    /// This equivalent to:
    ///
    /// ```ignore
    /// async fn collect(self) -> std::io::Result<BytesMut>;
    /// ```
    pub fn collect(self) -> Collect {
        Collect::new(self)
    }
}

impl Body {
    #[cfg(feature = "tokio")]
    pub(crate) fn poll_write_all_tcp(
        &mut self,
        _: &mut std::task::Context,
        tcp: &tokio::net::TcpStream,
    ) -> std::task::Poll<std::io::Result<()>> {
        use bytes::Buf;

        match &mut self.kind {
            BodyKind::Bytes(bytes) => {
                while !bytes.is_empty() {
                    let write = tcp.try_write(bytes)?;
                    bytes.advance(write);
                }
            }
            BodyKind::Tcp(_) => {
                panic!("[BUG] try to write tcp as tcp body");
            }
        }

        std::task::Poll::Ready(Ok(()))
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Body").finish()
    }
}

