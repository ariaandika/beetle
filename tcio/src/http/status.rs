/// HTTP Status Code.
use std::{
    fmt::{Debug, Display, Formatter},
    num::NonZeroU16,
};

/// HTTP Status Code.
#[derive(Clone, Copy)]
pub struct StatusCode(NonZeroU16);

impl StatusCode {
    /// Returns status code value, e.g: `200`.
    pub fn status(&self) -> u16 {
        self.0.get()
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.status_str())?;
        f.write_str(" ")?;
        f.write_str(self.message())
    }
}

impl Debug for StatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StatusCode").field(&self.0.get()).finish()
    }
}

impl Default for StatusCode {
    fn default() -> Self {
        Self::OK
    }
}

macro_rules! status_code_v3 {
    (
        $(
            $int:literal $id:ident $msg:literal;
        )*
    ) => {
        impl StatusCode {
            /// Returns status code and message as string slice, e.g: `"200 OK"`.
            pub const fn as_str(&self) -> &'static str {
                match self.0.get() {
                    $(
                        $int => concat!(stringify!($int)," ",$msg),
                    )*
                    // SAFETY: StatusCode value is privately constructed and immutable
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }
            }

            /// Returns status code as str, e.g: `"200"`.
            pub const fn status_str(&self) -> &'static str {
                match self.0.get() {
                    $(
                        $int => stringify!($int),
                    )*
                    // SAFETY: StatusCode value is privately constructed and immutable
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }
            }

            /// Returns status message, e.g: `"OK"`.
            pub const fn message(&self) -> &'static str {
                match self.0.get() {
                    $(
                        $int => $msg,
                    )*
                    // SAFETY: StatusCode value is privately constructed and immutable
                    _ => unsafe { std::hint::unreachable_unchecked() },
                }
            }

            $(
                pub const $id: Self = Self(unsafe { NonZeroU16::new_unchecked($int) });
            )*
        }
    };
}

status_code_v3! {
    200 OK "OK";
    400 BAD_REQUEST "Bad Request";
    404 NOT_FOUND "Not Found";
    405 METHOD_NOT_ALLOWED "Method Not Allowed";
    500 INTERNAL_SERVER_ERROR "Internal Server Error";
}

