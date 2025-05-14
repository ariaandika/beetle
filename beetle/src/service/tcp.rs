use bytes::{Buf, BufMut, Bytes, BytesMut};
use memchr::memmem::{self, FindIter, Finder, find_iter};
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

use super::HttpService;
use crate::{
    body::Body,
    common::ByteStr,
    http::{Header, Headers, Method, Version},
    request::{Parts, Request},
    response::{self, IntoResponse},
    service::Service,
};

fn to_io<E: Into<Box<dyn std::error::Error + Send + Sync>>>(err: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, err)
}

fn parse_str(val: &[u8]) -> Result<&str, io::Error> {
    std::str::from_utf8(val).map_err(to_io)
}

fn parse_int(val: &[u8]) -> Result<usize, io::Error> {
    parse_str(val)?.parse().map_err(to_io)
}

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
        WriteReady { body: Option<Body> },
        Write { body: Option<Body> },
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
                    let Some((method, path, version, header_offset)) = parse_request_line(buffer)? else {
                        let _ = buffer.try_reclaim(1024);
                        state.set(TcpState::IoReady);
                        continue;
                    };

                    let headers = &buffer[header_offset..];

                    let mut parser = HeaderParser::new(headers);
                    let mut headers_map = Vec::with_capacity(8);
                    let mut content_len = None;

                    for result in &mut parser {
                        let (key,val) = result;

                        if key.eq_ignore_ascii_case(b"content-length") {
                            content_len = Some(parse_int(val)?);
                        }

                        // TODO: prevent copy
                        headers_map.push(Header::new(
                            ByteStr::copy_from_str(parse_str(key)?),
                            Bytes::copy_from_slice(val),
                        ));
                    }

                    if !parser.complete() {
                        let _ = buffer.try_reclaim(1024);
                        state.set(TcpState::IoReady);
                        continue;
                    }

                    let body_offset = parser.offset();

                    let path_ptr = (path.as_ptr(), path.len());
                    let request_line = buffer.split_to(header_offset).freeze();

                    // `buffer` now contains [header..,body..]

                    // SAFETY: `buffer.split_to` will not move pointer and path was a `str`
                    let path = unsafe {
                        let path = std::slice::from_raw_parts(path_ptr.0, path_ptr.1);
                        ByteStr::from_utf8_unchecked(request_line.slice_ref(path))
                    };

                    // `body_offset` is offset started from `header_offset`,
                    // but `buffer` is already started from `header_offset`
                    buffer.advance(body_offset);

                    // `buffer` now contains [body..]

                    let body = buffer.split();

                    // `buffer` now empty

                    let headers = Headers::from_buffer(headers_map);
                    let parts = Parts::new(method, path, version, headers, <_>::default());
                    let body = Body::tcp(io.clone(), content_len.unwrap_or_default(), body);
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
                        let read = io.try_write(res_buffer)?;
                        res_buffer.advance(read);
                    }

                    ready!(Pin::new(body.as_mut().unwrap()).poll_write_all_tcp(cx, io)?);

                    #[cfg(feature = "log")]
                    log::trace!("request complete");
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

    fn into_poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        use TcpStateProject::*;

        let me = self.as_mut().project();
        let mut state = me.state;

        match state.as_mut().project() {
            IoReady | Parse | Inner { .. } | Cleanup => {
                #[cfg(feature = "log")]
                log::trace!("would block on read");
                return self.as_mut().project().io.poll_read_ready(cx);
            },
            WriteReady { body } => {
                #[cfg(feature = "log")]
                log::trace!("would block on write");
                let body = body.take();
                state.set(TcpState::WriteReady { body })
            },
            Write { body } => {
                #[cfg(feature = "log")]
                log::trace!("would block on write");
                let body = body.take();
                state.set(TcpState::WriteReady { body })
            },
        }

        Ready(Ok(()))
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
                let result = ready!(self.as_mut().into_poll_ready(cx));
                if let Err(_err) = result {
                    #[cfg(feature = "log")]
                    log::error!("{_err}");
                    return Ready(Err(()))
                };
                self.poll(cx)
            },
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {
                #[cfg(feature = "log")]
                log::trace!("interrupted");
                self.poll(cx)
            },
            Err(_err) => {
                #[cfg(feature = "log")]
                log::error!("{_err}");
                Ready(Err(()))
            }
        }
    }
}

// ===== Parser =====

/// Returns (method, path, version, header offset)
fn parse_request_line(
    buf: &[u8],
) -> io::Result<Option<(Method, &str, Version, usize)>> {
    let mut offset = 0;

    macro_rules! collect_until {
        ($e:ident => $b:expr) => {{
            let start = offset;
            loop {
                match buf.get(offset) {
                    Some($e) if $b => {
                        break &buf[start..offset];
                    }
                    Some(_) => {
                        offset += 1;
                    }
                    None => return Ok(None),
                }
            }
        }};
    }

    // NOTE: method

    let method = collect_until!(e => e.is_ascii_whitespace());
    let method = match method {
        b"GET" | b"get" => Method::GET,
        b"POST" | b"post" => Method::POST,
        b"PUT" | b"put" => Method::PUT,
        b"PATCH" | b"patch" => Method::PUT,
        b"DELETE" | b"delete" => Method::DELETE,
        b"HEAD" | b"head" => Method::HEAD,
        b"CONNECT" | b"connect" => Method::CONNECT,
        _ => return Err(to_io(format!("unknown method: {method:?}"))),
    };

    collect_until!(e => !e.is_ascii_whitespace());

    // NOTE: path

    let path = collect_until!(e => e.is_ascii_whitespace());
    let path = parse_str(path)?;

    collect_until!(e => !e.is_ascii_whitespace());

    // NOTE: version

    let version = collect_until!(e => e.is_ascii_whitespace());
    let version = match version {
        b"HTTP/1.0" => Version::V10,
        b"HTTP/1.1" => Version::V11,
        b"HTTP/2" => Version::V2,
        _ => return Err(to_io(format!("unknown http version: {version:?}"))),
    };

    match memmem::find(&buf[offset..], b"\r\n") {
        Some(ok) => Ok(Some((method, path, version, offset + ok + b"\r\n".len()))),
        None => Ok(None),
    }
}

pub struct HeaderParser<'a> {
    buf: &'a [u8],
    offset: usize,
    complete: bool,
    colsp: Finder<'static>,
    iter: FindIter<'a, 'static>,
}

impl<'a> HeaderParser<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self {
            buf,
            offset: 0,
            complete: false,
            colsp: Finder::new(b": "),
            iter: find_iter(buf, b"\r\n")
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn complete(&self) -> bool {
        self.complete
    }
}

impl<'a> Iterator for HeaderParser<'a> {
    type Item = (&'a [u8], &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.complete {
            return None;
        }

        let cr = self.iter.next()?;
        let kv = self.buf.get(self.offset..cr)?;

        if kv.is_empty() {
            self.complete = true;
            return None;
        }

        let colsp = self.colsp.find(kv)?;

        let key = kv.get(..colsp)?;
        let val = kv.get(colsp + 1..)?;

        self.offset = cr + 2;

        Some((key, val))
    }
}

