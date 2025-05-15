use bytes::{BufMut, Bytes, BytesMut};
use std::{
    io,
    pin::Pin,
    sync::{atomic::{AtomicUsize, Ordering}, Arc},
    task::{ready, Context, Poll},
};

use crate::{io::StreamReadExt, net::Socket};

fn exhausted() -> io::Error {
    io::Error::new(io::ErrorKind::QuotaExceeded, "request body exhausted")
}

#[derive(Debug)]
pub struct Body {
    content_len: usize,
    io: Option<Arc<Socket>>,

    /// How many content is read, including `self.buffer`
    /// and incoming bytes from `self.io`.
    read: AtomicUsize,

    /// May contains partially read body
    ///
    /// There is maybe already partially read body when parsing headers.
    /// If so, then this bytes contains it.
    ///
    /// We need to track read externally so no need mut reference,
    /// `self.read` also count `self.buffer` read.
    buffer: Bytes,
}

impl Body {
    /// Create empty [`Body`].
    pub fn empty() -> Self {
        Self {
            content_len: 0,
            io: None,
            read: AtomicUsize::new(0),
            buffer: Bytes::new()
        }
    }

    pub(crate) fn new(
        content_len: usize,
        io: Option<Arc<Socket>>,
        buffer: Bytes,
    ) -> Self {
        Self {
            content_len,
            io,
            read: AtomicUsize::new(buffer.len()),
            buffer,
        }
    }

    /// Returns maybe partially read body.
    ///
    /// There is maybe already partially read body when parsing headers.
    /// If so, then this bytes contains it.
    ///
    /// No cloning is done to ensure uniqueness.
    ///
    /// Note that subsequent read will not be written to this buffer.
    pub(crate) fn take_buffer(&mut self) -> Bytes {
        std::mem::take(&mut self.buffer)
    }

    /// Read all body and returns it as [`BytesMut`].
    pub fn collect(self) -> Collect {
        Collect {
            buffer: match self.buffer.try_into_mut() {
                Ok(ok) => ok,
                Err(ok) => ok.into(),
            },
            content_len: self.content_len,
            read: self.read.load(Ordering::Relaxed),
            io: self.io,
        }
    }

    /// Remaining content to be read.
    pub fn remaining(&self) -> usize {
        self.content_len - self.read.load(Ordering::Relaxed)
    }

    /// Returns `true` if there is still more content to be read.
    pub fn is_remaining(&self) -> bool {
        self.remaining() != 0
    }

    /// Poll read from underlying io.
    ///
    /// Note that there is maybe already partially read body when parsing headers.
    /// To access it, use [`take_buffer`][Self::take_buffer].
    ///
    /// # Errors
    ///
    /// If the underlying io is exhausted, an error is returned.
    pub(crate) fn poll_read_buf<B: BufMut>(
        &self,
        cx: &mut Context,
        buf: &mut B,
    ) -> Poll<io::Result<usize>> {
        match &self.io {
            Some(io) => io.poll_read_buf(cx, buf),
            None => Poll::Ready(Err(exhausted())),
        }
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug)]
pub struct Collect {
    buffer: BytesMut,
    content_len: usize,
    read: usize,
    io: Option<Arc<Socket>>,
}

impl Collect {
    pub fn remaining(&self) -> usize {
        self.content_len - self.read
    }

    /// Returns `true` if there is still more content to be read.
    pub fn is_remaining(&self) -> bool {
        self.remaining() != 0
    }
}

impl Future for Collect {
    type Output = io::Result<Bytes>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = self.get_mut();

        let Some(io) = &me.io else {
            return if me.buffer.is_empty() {
                Poll::Ready(Err(exhausted()))
            } else {
                Poll::Ready(Ok(std::mem::take(&mut me.buffer).freeze()))
            };
        };

        while me.content_len - me.read != 0 {
            me.read += ready!(io.poll_read_buf(cx, &mut me.buffer)?);
            // TODO: guard against overflow body,
            // will it contains subsequent request ?
        }

        Poll::Ready(Ok(std::mem::take(&mut me.buffer).freeze()))
    }
}
