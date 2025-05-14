use super::{Parts, Response};
use bytes::{BufMut, BytesMut};

/// perform a post write response
///
/// - add httpdate
/// - add content length
pub fn check(res: &mut Response) {
    res.parts.insert_header(
        b"content-length",
        itoa::Buffer::new().format(res.body.len()).as_bytes(),
    );
}

/// write http response parts into buffer
pub fn write(parts: &Parts, bytes: &mut BytesMut) {
    bytes.put_slice(parts.version.as_str().as_bytes());
    bytes.put_slice(b" ");
    bytes.put_slice(parts.status.as_str().as_bytes());
    bytes.put_slice(b"\r\n");
    bytes.put_slice(&parts.headers);
    bytes.extend_from_slice(b"\r\n");
}

