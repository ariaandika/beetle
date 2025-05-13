use memchr::memmem::{self, find_iter, FindIter, Finder};
use std::{
    io,
    str::{Utf8Error, from_utf8},
};

use crate::http::{Method, Version};

/// Returns (method, path, version, header offset)
pub fn parse_request_line(
    buf: &[u8],
) -> Result<Option<(Method, &str, Version, usize)>, ParseError> {
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
        _ => return Err(format!("unknown method: {method:?}").into()),
    };

    collect_until!(e => !e.is_ascii_whitespace());

    // NOTE: path

    let path = collect_until!(e => e.is_ascii_whitespace());
    let path = from_utf8(path)?;

    collect_until!(e => !e.is_ascii_whitespace());

    // NOTE: version

    let version = collect_until!(e => e.is_ascii_whitespace());
    let version = match version {
        b"HTTP/1.0" => Version::V10,
        b"HTTP/1.1" => Version::V11,
        b"HTTP/2" => Version::V2,
        _ => return Err(format!("unknown http version: {version:?}").into()),
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

/// Error that may returned from [`parse`].
#[derive(Debug)]
pub struct ParseError(String);

impl From<ParseError> for io::Error {
    fn from(value: ParseError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, value)
    }
}

impl From<String> for ParseError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<Utf8Error> for ParseError {
    fn from(value: Utf8Error) -> Self {
        Self(value.to_string())
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("failed to parse http, ")?;
        f.write_str(&self.0)
    }
}

