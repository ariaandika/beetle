use bytes::{BufMut, BytesMut};

use super::{Parts, Response};
use crate::headers::HeaderValue;

/// perform a post write response
///
/// - add httpdate
/// - add content length
pub fn validate(res: &mut Response) {
    // todo!("add httpdate")

    let mut b = itoa::Buffer::new();
    let content_len = b.format(res.body.content_len());
    res.parts.headers_mut().insert(
        "content-length",
        HeaderValue::try_copy_from_string(content_len).unwrap(),
    );
}

/// write http response parts into buffer
pub fn write(parts: &Parts, bytes: &mut BytesMut) {
    bytes.put_slice(parts.version().as_str().as_bytes());
    bytes.put_slice(b" ");
    bytes.put_slice(parts.status().as_str().as_bytes());
    bytes.put_slice(b"\r\n");
    for (name,value) in parts.headers().iter() {
        bytes.put_slice(name.as_str().as_bytes());
        bytes.put_slice(b": ");
        bytes.put_slice(value.as_bytes());
        bytes.put_slice(b"\r\n");
    }
    bytes.extend_from_slice(b"\r\n");
}

