//! Http Request and Response Body.
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll::{self, Ready},
        ready,
    },
};
use tokio::net::TcpStream;

pub struct Body {
    kind: BodyKind,
}

enum BodyKind {
    Empty,
    Bytes(Bytes),
    Tcp(TcpBody),
}

impl Body {
    pub(crate) fn empty() -> Self {
        Self {
            kind: BodyKind::Empty,
        }
    }

    pub(crate) fn bytes(bytes: impl Into<Bytes>) -> Self {
        Self {
            kind: BodyKind::Bytes(bytes.into()),
        }
    }

    pub(crate) fn tcp(shared: Arc<TcpStream>, content_len: usize, buffer: BytesMut) -> Self {
        Self {
            kind: BodyKind::Tcp(TcpBody {
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
            BodyKind::Bytes(b) => b.len(),
            BodyKind::Tcp(b) => b.content_len,
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
        Collect { body: self }
    }

    pub(crate) fn is_remaining(&self) -> bool {
        match &self.kind {
            BodyKind::Empty => false,
            BodyKind::Bytes(bytes) => !bytes.is_empty(),
            BodyKind::Tcp(body) => body.is_remaining(),
        }
    }

    #[allow(unused, reason = "used by form later")]
    pub(crate) fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        match &mut self.kind {
            BodyKind::Empty => Ready(Ok(())),
            BodyKind::Bytes(_) => Ready(Ok(())),
            BodyKind::Tcp(body) => Pin::new(body).poll_read(cx),
        }
    }

    pub(crate) fn poll_write_all_tcp(
        mut self: Pin<&mut Self>,
        _: &mut Context,
        io: &TcpStream,
    ) -> Poll<io::Result<()>> {
        while self.is_remaining() {
            match &mut self.kind {
                BodyKind::Empty => {}
                BodyKind::Bytes(b) => {
                    let read = io.try_write(b)?;
                    b.advance(read);
                }
                BodyKind::Tcp(_) => panic!("cannot write tcp kind of `Body`"),
            }
        }

        Ready(Ok(()))
    }
}

impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Body").field(&self.content_len()).finish()
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
            BodyKind::Bytes(b) => return Ready(Ok(std::mem::take(b))),
            BodyKind::Tcp(ok) => ok,
        };

        while body.read_len < body.content_len {
            ready!(Pin::new(&mut *body).poll_read(cx)?);
        }

        Ready(Ok(std::mem::take(&mut body.buffer).freeze()))
    }
}

struct TcpBody {
    io: Arc<TcpStream>,
    content_len: usize,
    read_len: usize,
    buffer: BytesMut,
}

fn io_err<E: Into<Box<dyn std::error::Error + Send + Sync>>>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}

impl TcpBody {
    fn is_remaining(&self) -> bool {
        self.read_len < self.content_len
    }

    fn is_end_stream(&self) -> bool {
        self.read_len >= self.content_len
    }

    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        if self.is_end_stream() {
            return Ready(Err(io_err("body exhausted")));
        }

        ready!(self.io.poll_read_ready(cx)?);

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

impl Buf for Body {
    fn remaining(&self) -> usize {
        match &self.kind {
            BodyKind::Empty => 0,
            BodyKind::Bytes(b) => b.remaining(),
            BodyKind::Tcp(b) => b.content_len - b.read_len,
        }
    }

    fn chunk(&self) -> &[u8] {
        match &self.kind {
            BodyKind::Empty => &[],
            BodyKind::Bytes(b) => b.chunk(),
            BodyKind::Tcp(b) => b.buffer.chunk(),
        }
    }

    fn advance(&mut self, cnt: usize) {
        match &mut self.kind {
            BodyKind::Empty => {}
            BodyKind::Bytes(b) => b.advance(cnt),
            BodyKind::Tcp(b) => b.buffer.advance(cnt),
        }
    }
}

