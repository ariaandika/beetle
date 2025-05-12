//! request and response body struct
use bytes::{Bytes, BytesMut};
use std::{
    io,
    pin::{Pin, pin},
    sync::{Arc, Mutex},
    task::{
        Context,
        Poll::{self, *},
        ready,
    },
};
use tokio::net::TcpStream;

/// http request body
//
// lock based body reader
pub struct Body {
    shared: Arc<Mutex<TcpStream>>,
    content_len: usize,
    read_len: usize,
    buffer: BytesMut,
}

impl Body {
    pub(crate) fn new(shared: Arc<Mutex<TcpStream>>, content_len: usize, buffer: BytesMut) -> Self {
        Self {
            shared,
            content_len,
            read_len: 0,
            buffer,
        }
    }

    /// return content-length
    ///
    /// chunked content is not yet supported
    pub fn content_len(&self) -> usize {
        self.content_len
    }

    // /// consume body into [`BytesMut`]
    // pub fn bytes_mut(self) -> StreamFuture<BytesMut> {
    //     let Some(BodyChan { stream, buffer, content_len, }) = self.chan else {
    //         // should if content length is missing or invalid,
    //         // an io error [`io::ErrorKind::InvalidData`] is returned ?
    //         return StreamFuture::exact(BytesMut::new())
    //     };
    //     let read = buffer.len();
    //     let read_left = content_len.saturating_sub(read);
    //     if read_left == 0 {
    //         return StreamFuture::exact(buffer)
    //     }
    //     stream.read_exact(read, read_left, buffer)
    // }

    // /// consume body into [`Bytes`]
    // ///
    // /// this is utility function that propagate [`Body::bytes_mut`]
    // pub async fn bytes(self) -> io::Result<Bytes> {
    //     Ok(self.bytes_mut().await?.freeze())
    // }
}

impl Body {
    pub fn collect(self) {
        
    }

    pub(crate) fn poll_read(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        if self.read_len >= self.content_len {
            return Ready(Err(io::Error::new(
                io::ErrorKind::QuotaExceeded,
                "content length reached",
            )));
        }

        let me = self.get_mut();
        let lock = match me.shared.try_lock() {
            Ok(ok) => ok,
            Err(_) => {
                return Ready(Err(io::Error::new(
                    io::ErrorKind::ResourceBusy,
                    "unable to lock for body read",
                )));
            }
        };

        ready!(pin!(lock.readable()).poll(cx)?);
        let read = lock.try_read_buf(&mut me.buffer)?;

        me.read_len += read;

        Ready(Ok(()))
    }
}

pub struct Collect {
    body: Body,
}

impl Future for Collect {
    type Output = io::Result<BytesMut>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = &mut self.get_mut().body;
        while me.read_len < me.content_len {
            ready!(Pin::new(&mut *me).poll_read(cx)?);
        }
        Ready(Ok(std::mem::take(&mut me.buffer)))
    }
}

#[derive(Default)]
pub enum ResBody {
    #[default]
    Empty,
    Bytes(Bytes),
}

impl ResBody {
    /// return buffer length
    pub fn len(&self) -> usize {
        match self {
            ResBody::Empty => 0,
            ResBody::Bytes(b) => b.len(),
        }
    }

    /// return is buffer length empty
    pub fn is_empty(&self) -> bool {
        match self {
            ResBody::Empty => true,
            ResBody::Bytes(b) => b.is_empty(),
        }
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



impl std::fmt::Debug for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Body").field(&self.content_len).finish()
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

