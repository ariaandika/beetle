//! Http Request and Response Body.
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{
    io,
    pin::{pin, Pin},
    sync::Arc,
    task::{
        ready, Context, Poll::{self, Ready}
    },
};
use tokio::net::TcpStream;

/// Http Request Body.
pub struct Body {
    io: Arc<TcpStream>,
    content_len: Option<usize>,
    read_len: usize,
    buffer: BytesMut,
}

impl Body {
    pub(crate) fn new(shared: Arc<TcpStream>, content_len: Option<usize>, buffer: BytesMut) -> Self {
        Self {
            io: shared,
            content_len,
            read_len: 0,
            buffer,
        }
    }

    /// Returns `Content-length` if any.
    pub fn content_len(&self) -> Option<usize> {
        self.content_len
    }
}

impl Body {
    pub(crate) fn poll_read(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        let Some(content_len) = self.content_len else {
            return Ready(Err(io::Error::new(
                io::ErrorKind::QuotaExceeded,
                "request contains no body",
            )));
        };

        if self.read_len >= content_len {
            return Ready(Err(io::Error::new(
                io::ErrorKind::QuotaExceeded,
                "content length reached",
            )));
        }

        let me = self.get_mut();
        ready!(pin!(me.io.readable()).poll(cx)?);

        let read = {
            let buf = me.buffer.chunk_mut();
            let buf = unsafe {
                &mut *(buf.as_uninit_slice_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8])
            };
            me.io.try_read(buf)?
        };
        unsafe { me.buffer.advance_mut(read) };

        me.read_len += read;

        Ready(Ok(()))
    }

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

/// Futures returned from [`Body::collect`].
#[derive(Debug)]
#[must_use = "`Future` does nothing unless polled or .awaited"]
pub struct Collect {
    body: Body,
}

impl Future for Collect {
    type Output = io::Result<BytesMut>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = &mut self.get_mut().body;

        let Some(content_len) = me.content_len else {
            return Ready(Ok(BytesMut::new()));
        };

        while me.read_len < content_len {
            ready!(Pin::new(&mut *me).poll_read(cx)?);
        }
        Ready(Ok(std::mem::take(&mut me.buffer)))
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Body").field(&self.content_len).finish()
    }
}

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

