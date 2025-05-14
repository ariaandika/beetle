use bytes::BytesMut;
use std::{
    io,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

#[cfg(feature = "tokio")]
type ArcTcp = std::sync::Arc<tokio::net::TcpStream>;

#[cfg(not(feature = "tokio"))]
type ArcTcp = ();

pub struct TcpBody {
    io: ArcTcp,
    content_len: usize,
    read_len: usize,
    buffer: BytesMut,
}

impl TcpBody {
    #[cfg(feature = "tokio")]
    pub(crate) fn new(
        io: ArcTcp,
        content_len: usize,
        buffer: BytesMut,
    ) -> Self {
        Self {
            io,
            content_len,
            read_len: 0,
            buffer,
        }
    }

    pub(crate) fn content_len(&self) -> usize {
        self.content_len
    }

    pub(crate) fn remaining(&self) -> usize {
        self.content_len - self.read_len
    }

    pub fn buffer_mut(&mut self) -> &mut BytesMut {
        &mut self.buffer
    }

    pub(crate) fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        #[cfg(feature = "tokio")]
        {
            use std::task::{ready, Poll::Ready};
            use bytes::BufMut;

            if self.read_len >= self.content_len {
                return Ready(Err(io::Error::new(io::ErrorKind::InvalidData, "body exhausted")));
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

        #[cfg(not(feature = "tokio"))]
        {
            let _ = (&mut self.io,cx);
            panic!("runtime disabled")
        }
    }
}

