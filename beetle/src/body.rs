//! Http Request and Response Body.
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{
    io,
    pin::{Pin, pin},
    sync::Arc,
    task::{
        Context,
        Poll::{self, Ready},
        ready,
    },
};
use tokio::net::TcpStream;

/// Http Request Body.
pub struct Body {
    kind: BodyKind
}

enum BodyKind {
    Empty,
    Exact(Bytes),
    Arc(ArcBody),
}

impl Body {
    pub fn empty() -> Self {
        Self {
            kind: BodyKind::Empty,
        }
    }

    pub fn exact(bytes: Bytes) -> Self {
        Self {
            kind: BodyKind::Exact(bytes),
        }
    }

    pub(crate) fn new(shared: Arc<TcpStream>, content_len: usize, buffer: BytesMut) -> Self {
        Self {
            kind: BodyKind::Arc(ArcBody {
                io: shared,
                content_len,
                read_len: 0,
                buffer,
            }),
        }
    }

    /// Returns body length.
    pub fn content_len(&self) -> usize {
        match &self.kind {
            BodyKind::Empty => 0,
            BodyKind::Exact(b) => b.len(),
            BodyKind::Arc(b) => b.content_len,
        }
    }
}

impl Body {
    /// Read all body into [`BytesMut`].
    ///
    /// This equivalent to:
    ///
    /// ```ignore
    /// async fn collect(self) -> std::io::Result<BytesMut>;
    /// ```
    pub fn collect(self) -> Collect {
        Collect { body: self }
    }
}

struct ArcBody {
    io: Arc<TcpStream>,
    content_len: usize,
    read_len: usize,
    buffer: BytesMut,
}

impl ArcBody {
    pub(crate) fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        if self.read_len >= self.content_len {
            return Ready(Err(io::Error::new(
                io::ErrorKind::QuotaExceeded,
                "body exhausted",
            )));
        }

        ready!(pin!(self.io.readable()).poll(cx)?);

        let read = {
            let buf = self.buffer.chunk_mut();
            let buf = unsafe {
                &mut *(buf.as_uninit_slice_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8])
            };
            self.io.try_read(buf)?
        };
        unsafe { self.buffer.advance_mut(read) };

        self.read_len += read;

        Ready(Ok(()))
    }
}

/// Futures returned from [`Body::collect`].
#[derive(Debug)]
#[must_use = "`Future` does nothing unless polled or .awaited"]
pub struct Collect {
    body: Body,
}

impl Future for Collect {
    type Output = io::Result<Bytes>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let body = match &mut self.body.kind {
            BodyKind::Empty => return Ready(Ok(<_>::default())),
            BodyKind::Exact(b) => return Ready(Ok(std::mem::take(b))),
            BodyKind::Arc(ok) => ok,
        };

        while body.read_len < body.content_len {
            ready!(Pin::new(&mut *body).poll_read(cx)?);
        }

        Ready(Ok(std::mem::take(&mut body.buffer).freeze()))
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Body").field(&self.content_len()).finish()
    }
}


// ===== Response Body =====

#[derive(Default)]
pub enum ResBody {
    #[default]
    Empty,
    Bytes(Bytes),
}

impl ResBody {
    /// Returns buffer length.
    pub fn len(&self) -> usize {
        match self {
            ResBody::Empty => 0,
            ResBody::Bytes(b) => b.len(),
        }
    }

    /// Returns `true` if buffer is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            ResBody::Empty => true,
            ResBody::Bytes(b) => b.is_empty(),
        }
    }

    pub(crate) fn poll_write_all_tcp(&mut self, _: &mut Context, io: &TcpStream) -> Poll<io::Result<()>> {
        while !self.is_empty() {
            match self {
                ResBody::Empty => {},
                ResBody::Bytes(buf) => {
                    let read = io.try_write(buf)?;
                    buf.advance(read);
                },
            }
        }
        Poll::Ready(Ok(()))
    }
}

impl AsRef<[u8]> for ResBody {
    fn as_ref(&self) -> &[u8] {
        match self {
            ResBody::Empty => &[],
            ResBody::Bytes(bytes) => bytes.as_ref(),
        }
    }
}

impl From<&'static [u8]> for ResBody {
    fn from(value: &'static [u8]) -> Self {
        Self::Bytes(Bytes::from_static(value))
    }
}

impl From<Bytes> for ResBody {
    fn from(value: Bytes) -> Self {
        Self::Bytes(value)
    }
}

impl From<Vec<u8>> for ResBody {
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value.into())
    }
}

impl From<String> for ResBody {
    fn from(value: String) -> Self {
        Self::Bytes(value.into_bytes().into())
    }
}

impl std::fmt::Debug for ResBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.debug_tuple("ResBody").field(&"Empty").finish(),
            Self::Bytes(b) => f.debug_tuple("ResBody").field(b).finish(),
        }
    }
}

