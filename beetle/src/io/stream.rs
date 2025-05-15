use bytes::{Buf, BufMut};
use std::{
    io,
    mem::MaybeUninit,
    task::{Context, Poll, ready},
};

pub trait StreamRead {
    fn try_read(&self, buf: &mut [u8]) -> io::Result<usize>;

    fn poll_read_ready(&self, cx: &mut Context) -> Poll<io::Result<()>>;
}

pub trait StreamWrite {
    fn try_write(&self, buf: &[u8]) -> io::Result<usize>;

    fn poll_write_ready(&self, cx: &mut Context) -> Poll<io::Result<()>>;
}

pub trait StreamReadExt: StreamRead {
    fn poll_read(&self, cx: &mut Context, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        let result = self.try_read(buf);

        if matches!(result.as_ref(),Err(err) if err.kind() == io::ErrorKind::WouldBlock) {
            ready!(self.poll_read_ready(cx)?);
            return self.poll_read(cx, buf);
        }

        Poll::Ready(result)
    }

    fn poll_read_buf<B>(&self, cx: &mut Context, buf: &mut B) -> Poll<io::Result<usize>>
    where
        B: BufMut,
    {
        if !buf.has_remaining_mut() {
            return Poll::Ready(Ok(0));
        }

        let read = {
            let dst = buf.chunk_mut();
            let dst = unsafe { &mut *(dst as *mut _ as *mut [MaybeUninit<u8>] as *mut [u8]) };
            ready!(self.poll_read(cx, dst)?)
        };

        // SAFETY: This is guaranteed to be the number of initialized (and read) bytes
        // provided by `StreamRead::try_read`.
        unsafe {
            buf.advance_mut(read);
        }

        Poll::Ready(Ok(read))
    }
}

impl<S: StreamRead> StreamReadExt for S { }

pub trait StreamWriteExt: StreamWrite {
    fn poll_write<B: Buf>(&self, cx: &mut Context, buf: &mut B) -> Poll<io::Result<usize>> {
        let result = self.try_write(buf.chunk());

        if matches!(result.as_ref(),Err(err) if err.kind() == io::ErrorKind::WouldBlock) {
            ready!(self.poll_write_ready(cx)?);
            return self.poll_write(cx, buf);
        }

        let read = result?;
        buf.advance(read);
        Poll::Ready(Ok(read))
    }

    fn poll_write_all<B: Buf>(&self, cx: &mut Context, buf: &mut B) -> Poll<io::Result<()>> {
        while Buf::has_remaining(&buf) {
            ready!(self.poll_write(cx, buf));
        }

        Poll::Ready(Ok(()))
    }
}

impl<S: StreamWrite> StreamWriteExt for S { }

#[cfg(feature = "tokio")]
mod rt_tokio {
    use tokio::net::{TcpStream, UnixStream};

    use super::*;

    impl StreamRead for TcpStream {
        fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
            TcpStream::try_read(self, buf)
        }

        fn poll_read_ready(&self, cx: &mut Context) -> Poll<io::Result<()>> {
            TcpStream::poll_read_ready(self, cx)
        }
    }

    impl StreamWrite for TcpStream {
        fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
            TcpStream::try_write(self, buf)
        }

        fn poll_write_ready(&self, cx: &mut Context) -> Poll<io::Result<()>> {
            TcpStream::poll_write_ready(self, cx)
        }
    }

    impl StreamRead for UnixStream {
        fn try_read(&self, buf: &mut [u8]) -> io::Result<usize> {
            UnixStream::try_read(self, buf)
        }

        fn poll_read_ready(&self, cx: &mut Context) -> Poll<io::Result<()>> {
            UnixStream::poll_read_ready(self, cx)
        }
    }

    impl StreamWrite for UnixStream {
        fn try_write(&self, buf: &[u8]) -> io::Result<usize> {
            UnixStream::try_write(self, buf)
        }

        fn poll_write_ready(&self, cx: &mut Context) -> Poll<io::Result<()>> {
            UnixStream::poll_write_ready(self, cx)
        }
    }
}

