use bytes::{Buf, BufMut, BytesMut};
use std::{
    io,
    pin::{Pin, pin},
    str::from_utf8,
    sync::Arc,
    task::{
        Context,
        Poll::{self, *},
        ready,
    },
};
use tokio::net::TcpStream;

use super::HttpService;
use crate::{
    ResBody,
    body::Body,
    request::{self, Request},
    response::{self, IntoResponse},
    service::Service,
};

#[derive(Debug, Clone)]
pub struct TcpService<S> {
    inner: S,
}

impl<S> TcpService<S> {
    pub fn new(inner: S) -> TcpService<S> {
        TcpService { inner }
    }
}

impl<S> Service<TcpStream> for TcpService<S>
where
    S: HttpService + Clone,
{
    type Response = ();

    type Error = ();

    type Future = TcpFuture<S, S::Future>;

    fn call(&self, io: TcpStream) -> Self::Future {
        #[cfg(feature = "log")]
        log::trace!("connection open");
        TcpFuture {
            inner: self.inner.clone(),
            buffer: BytesMut::with_capacity(1024),
            res_buffer: BytesMut::with_capacity(1024),
            io: Arc::new(io),
            state: TcpState::IoReady,
        }
    }
}

pin_project_lite::pin_project! {
    #[project = TcpStateProject]
    enum TcpState<Fut> {
        IoReady,
        Parse,
        Inner { #[pin] future: Fut },
        WriteReady { body: Option<ResBody> },
        Write { body: Option<ResBody> },
        Cleanup,
    }
}

pin_project_lite::pin_project! {
    #[project = TcpProject]
    pub struct TcpFuture<S,F> {
        inner: S,
        buffer: BytesMut,
        res_buffer: BytesMut,
        io: Arc<TcpStream>,
        #[pin]
        state: TcpState<F>,
    }
}

impl<S> TcpFuture<S,S::Future>
where
    S: Service<Request>,
    S::Response: IntoResponse,
    S::Error: IntoResponse,
{
    fn try_poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        use TcpStateProject::*;

        let TcpProject {
            inner,
            buffer,
            res_buffer,
            io,
            mut state,
        } = self.as_mut().project();

        loop {
            match state.as_mut().project() {
                IoReady => {
                    ready!(pin!(io.readable()).poll(cx)?);
                    let read = {
                        let buf = buffer.chunk_mut();
                        let buf = unsafe {
                            &mut *(buf.as_uninit_slice_mut() as *mut [std::mem::MaybeUninit<u8>] as *mut [u8])
                        };
                        io.try_read(buf)?
                    };
                    unsafe { buffer.advance_mut(read) };
                    if read == 0 {
                        return Ready(Ok(()));
                    }
                    state.set(TcpState::Parse);
                }
                Parse => {
                    let parts = match request::parse(buffer) {
                        Ok(Some(ok)) => ok,
                        Ok(None) => {
                            #[cfg(feature = "log")]
                            log::debug!("buffer should be unique to reclaim: {:?}",buffer.try_reclaim(1024));
                            state.set(TcpState::IoReady);
                            continue;
                        },
                        Err(err) => return Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            err,
                        )))
                    };

                    let content_len = parts
                        .headers()
                        .iter()
                        .find_map(|header| (header.name == "content-length").then_some(header.value.as_ref()))
                        .and_then(|e| from_utf8(e).ok()?.parse().ok());

                    let Some(len) = content_len else {
                        return Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "error: no content length header",
                        )));
                    };

                    let body = Body::new(io.clone(), len, buffer.split());
                    let request = Request::from_parts(parts,body);
                    let future = inner.call(request);
                    state.set(TcpState::Inner { future });
                }
                Inner { future } => {
                    let mut response = ready!(future.poll(cx)).into_response();
                    response::check(&mut response);
                    let (parts,body) = response.into_parts();
                    response::write(&parts, res_buffer);
                    state.set(TcpState::WriteReady { body: Some(body) });
                }
                WriteReady { body } => {
                    ready!(io.poll_write_ready(cx)?);
                    let body = body.take();
                    state.set(TcpState::Write { body });
                },
                Write { body } => {
                    while res_buffer.has_remaining() {
                        let read = io.try_write(&res_buffer)?;
                        res_buffer.advance(read);
                    }

                    ready!(body.as_mut().unwrap().poll_write_all_tcp(cx, io)?);

                    state.set(TcpState::Cleanup);
                },
                Cleanup => {
                    // this state will make sure all shared buffer is dropped
                    res_buffer.clear();
                    buffer.clear();

                    if !buffer.try_reclaim(1024) {
                        #[cfg(feature = "log")]
                        log::trace!("failed to reclaim buffer");
                    }

                    if !res_buffer.try_reclaim(1024) {
                        #[cfg(feature = "log")]
                        log::trace!("failed to reclaim res_buffer");
                    }

                    state.set(TcpState::IoReady);
                },
            }
        }
    }

    fn into_poll_ready(mut self: Pin<&mut Self>) -> Pin<&mut Self> {
        use TcpStateProject::*;

        let me = self.as_mut().project();
        let mut state = me.state;

        match state.as_mut().project() {
            IoReady => state.set(TcpState::IoReady),
            Parse => state.set(TcpState::IoReady),
            Inner { .. } => state.set(TcpState::IoReady),
            WriteReady { body } => {
                let body = body.take();
                state.set(TcpState::WriteReady { body })
            },
            Write { body } => {
                let body = body.take();
                state.set(TcpState::WriteReady { body })
            },
            Cleanup => state.set(TcpState::IoReady),
        }

        self
    }
}


impl<S> Future for TcpFuture<S, S::Future>
where
    S: Service<Request>,
    S::Response: IntoResponse,
    S::Error: IntoResponse,
{
    type Output = Result<(), ()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(self.as_mut().try_poll(cx)) {
            Ok(()) => {
                #[cfg(feature = "log")]
                log::trace!("connection closed");
                Ready(Ok(()))
            },
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                #[cfg(feature = "log")]
                log::trace!("would block");
                self.into_poll_ready().poll(cx)
            },
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {
                #[cfg(feature = "log")]
                log::trace!("interrupted");
                self.into_poll_ready().poll(cx)
            },
            Err(_err) => {
                #[cfg(feature = "log")]
                log::error!("{_err}");
                Ready(Err(()))
            }
        }
    }
}

