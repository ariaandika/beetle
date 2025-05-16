use bytes::{Buf, Bytes, BytesMut};
use memchr::memmem::{self, FindIter, Finder, find_iter};
use std::{
    hint, io,
    pin::Pin,
    sync::Arc,
    task::{
        Context,
        Poll::{self, Ready},
        ready,
    },
};

use super::HttpService;
use crate::{
    common::ByteStr,
    headers::{Header, Headers},
    http::{Method, Version},
    io::{StreamReadExt, StreamWriteExt},
    net::Socket,
    request::{self, Parts, Request},
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

impl<S> Service<Socket> for TcpService<S>
where
    S: HttpService + Clone,
{
    type Response = ();

    type Error = ();

    type Future = TcpFuture<S, S::Future>;

    fn call(&self, io: Socket) -> Self::Future {
        #[cfg(feature = "log")]
        log::trace!("connection open");
        TcpFuture {
            inner: self.inner.clone(),
            buffer: BytesMut::with_capacity(1024),
            res_buffer: BytesMut::with_capacity(1024),
            io: Arc::new(io),
            phase: TcpPhase::Read,
        }
    }
}

pin_project_lite::pin_project! {
    #[project = TcpPhaseProject]
    #[project_replace = TcpReplace]
    enum TcpPhase<Fut> {
        Read,
        Parse,
        Inner { #[pin] future: Fut },
        ResponseData { body: response::Body },
        Write { body: response::Body, data: Bytes },
        Cleanup,
    }
}

pin_project_lite::pin_project! {
    #[project = TcpProject]
    pub struct TcpFuture<S,F> {
        inner: S,
        buffer: BytesMut,
        res_buffer: BytesMut,
        io: Arc<Socket>,
        #[pin]
        phase: TcpPhase<F>,
    }
}

impl<S> TcpFuture<S,S::Future>
where
    S: Service<Request>,
    S::Response: IntoResponse,
    S::Error: IntoResponse,
{
    fn try_poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
        use TcpPhaseProject::*;

        let TcpProject {
            inner,
            buffer,
            res_buffer,
            io,
            mut phase,
        } = self.as_mut().project();

        loop {
            match phase.as_mut().project() {
                Read => {
                    let read = ready!(io.poll_read_buf(cx, buffer)?);
                    if read == 0 {
                        return Ready(Ok(()));
                    }
                    phase.set(TcpPhase::Parse);
                }
                Parse => {
                    let Some((method, path, version, header_offset)) = parse_request_line(buffer)? else {
                        let _ = buffer.try_reclaim(1024);
                        phase.set(TcpPhase::Read);
                        continue;
                    };

                    let headers = &buffer[header_offset..];

                    let mut parser = HeaderParser::new(headers);
                    let mut headers_map = Vec::with_capacity(8);
                    let mut content_len = 0;

                    for result in &mut parser {
                        let (key,val) = result;

                        if key.eq_ignore_ascii_case(b"content-length") {
                            content_len = parse_int(val)?;
                        }

                        // TODO: prevent copy
                        headers_map.push(Header::new(
                            ByteStr::copy_from_str(parse_str(key)?),
                            Bytes::copy_from_slice(val),
                        ));
                    }

                    if !parser.complete() {
                        phase.set(TcpPhase::Read);
                        continue;
                    }

                    let body_offset = parser.offset();

                    let path_ptr = (path.as_ptr(), path.len());
                    let request_line = buffer.split_to(header_offset).freeze();

                    // `buffer` now contains [headers..,body..]

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
                    let body = request::Body::new(content_len, Some(io.clone()), body.freeze());
                    let request = Request::from_parts(parts, body);

                    let future = inner.call(request);
                    phase.set(TcpPhase::Inner { future });
                }
                Inner { future } => {
                    let mut response = ready!(future.poll(cx)).into_response();
                    response::validate(&mut response);
                    let (parts,body) = response.into_parts();
                    response::write(&parts, res_buffer);
                    phase.set(TcpPhase::ResponseData { body });
                }
                ResponseData { body } => {
                    let data = ready!(body.poll_data(cx)?);
                    let TcpReplace::ResponseData { body } = phase.as_mut().project_replace(TcpPhase::Cleanup) else {
                        // SAFETY: we are in match arm of it
                        unsafe { hint::unreachable_unchecked() }
                    };
                    phase.set(TcpPhase::Write { body, data });
                },
                Write { body, data } => {
                    ready!(io.poll_write_all(cx, data)?);

                    if body.is_end_stream() {
                        phase.set(TcpPhase::Cleanup);
                    } else {
                        let TcpReplace::Write { body, data } = phase.as_mut().project_replace(TcpPhase::Cleanup) else {
                            // SAFETY: we are in match arm of it
                            unsafe { hint::unreachable_unchecked() }
                        };
                        debug_assert!(data.is_empty());
                        phase.set(TcpPhase::ResponseData { body });
                    }
                },
                Cleanup => {
                    // this state will make sure all shared buffer is dropped
                    res_buffer.clear();
                    buffer.clear();

                    buffer.reserve(1024);
                    res_buffer.reserve(1024);

                    phase.set(TcpPhase::Read);
                },
            }
        }
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
///
/// Cant immediately build `Parts` because `path` require access to `Bytes` and parser cannot have
/// access to `Bytes` because the buffer is a `BytesMut` and cant be `freeze` because read maybe
/// incomplete and cannot be converted back into `BytesMut` without copy.
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

struct HeaderParser<'a> {
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

