use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "tokio")]
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::{TcpStream, UnixStream},
};

use crate::io::{StreamRead, StreamWrite};

/// An either `TcpStream` or `Socket`, which implement
/// `AsyncRead` and `AsyncWrite` transparently.
///
/// Require `tokio` feature, otherwise panic at runtime.
pub struct Socket {
    kind: Kind,
}

enum Kind {
    #[cfg(feature = "tokio")]
    TokioTcp(TcpStream),
    #[cfg(all(feature = "tokio", unix))]
    TokioUnixSocket(UnixStream),
}

impl StreamRead for Socket {
    fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
        match &self.kind {
            #[cfg(feature = "tokio")]
            Kind::TokioTcp(t) => t.try_read(buf),
            #[cfg(all(feature = "tokio", unix))]
            Kind::TokioUnixSocket(u) => u.try_read(buf),
            #[cfg(not(feature = "tokio"))]
            _ => Ok(0)
        }
    }

    fn poll_read_ready(&self, cx: &mut Context) -> Poll<io::Result<()>> {
        match &self.kind {
            #[cfg(feature = "tokio")]
            Kind::TokioTcp(t) => t.poll_read_ready(cx),
            #[cfg(all(feature = "tokio", unix))]
            Kind::TokioUnixSocket(u) => u.poll_read_ready(cx),
            #[cfg(not(feature = "tokio"))]
            _ => Poll::Ready(Ok(())),
        }
    }
}

impl StreamWrite for Socket {
    fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
        match &self.kind {
            #[cfg(feature = "tokio")]
            Kind::TokioTcp(t) => t.try_write(buf),
            #[cfg(all(feature = "tokio", unix))]
            Kind::TokioUnixSocket(u) => u.try_write(buf),
            #[cfg(not(feature = "tokio"))]
            _ => Ok(0),
        }
    }

    fn poll_write_ready(&self, cx: &mut Context) -> Poll<io::Result<()>> {
        match &self.kind {
            #[cfg(feature = "tokio")]
            Kind::TokioTcp(t) => t.poll_write_ready(cx),
            #[cfg(all(feature = "tokio", unix))]
            Kind::TokioUnixSocket(u) => u.poll_write_ready(cx),
            #[cfg(not(feature = "tokio"))]
            _ => Poll::Ready(Ok(())),
        }
    }
}

#[cfg(feature = "tokio")]
impl AsyncRead for Socket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match &mut self.kind {
            Kind::TokioTcp(t) => Pin::new(t).poll_read(cx, buf),
            #[cfg(unix)]
            Kind::TokioUnixSocket(u) => Pin::new(u).poll_read(cx, buf),
        }
    }
}

#[cfg(feature = "tokio")]
impl AsyncWrite for Socket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut self.kind {
            Kind::TokioTcp(t) => Pin::new(t).poll_write(cx, buf),
            #[cfg(unix)]
            Kind::TokioUnixSocket(u) => Pin::new(u).poll_write(cx, buf),
        }
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        match &mut self.kind {
            Kind::TokioTcp(t) => Pin::new(t).poll_write_vectored(cx, bufs),
            #[cfg(unix)]
            Kind::TokioUnixSocket(u) => Pin::new(u).poll_write_vectored(cx, bufs),
        }
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.kind {
            Kind::TokioTcp(t) => Pin::new(t).poll_shutdown(cx),
            #[cfg(unix)]
            Kind::TokioUnixSocket(u) => Pin::new(u).poll_shutdown(cx),
        }
    }
}

#[cfg(feature = "tokio")]
impl From<TcpStream> for Socket {
    fn from(value: TcpStream) -> Self {
        Self {
            kind: Kind::TokioTcp(value),
        }
    }
}

#[cfg(feature = "tokio")]
impl From<UnixStream> for Socket {
    fn from(value: UnixStream) -> Self {
        Self {
            kind: Kind::TokioUnixSocket(value),
        }
    }
}

impl std::fmt::Debug for Socket {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            #[cfg(feature = "tokio")]
            Kind::TokioTcp(tcp) => std::fmt::Debug::fmt(&tcp, _f),
            #[cfg(all(feature = "tokio", unix))]
            Kind::TokioUnixSocket(unix) => std::fmt::Debug::fmt(&unix, _f),
            #[cfg(not(feature = "tokio"))]
            _ => Ok(()),
        }
    }
}
