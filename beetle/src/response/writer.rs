use super::{Parts, Response};
use bytes::{BufMut, BytesMut};

/// perform a post write response
///
/// - add httpdate
/// - add content length
pub fn validate(res: &mut Response) {
    // todo!("add httpdate")

    let mut b = itoa::Buffer::new();
    let content_len = b.format(res.body.content_len());
    res.parts.insert_header(
        b"content-length",
        content_len.as_bytes(),
    );
}

/// write http response parts into buffer
pub fn write(parts: &Parts, bytes: &mut BytesMut) {
    bytes.put_slice(parts.version().as_str().as_bytes());
    bytes.put_slice(b" ");
    bytes.put_slice(parts.status().as_str().as_bytes());
    bytes.put_slice(b"\r\n");
    // bytes.put_slice(&parts.headers());
    bytes.extend_from_slice(b"\r\n");
}

