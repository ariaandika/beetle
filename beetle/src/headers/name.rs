use std::hash::Hasher;

use crate::common::ByteStr;

pub struct HeaderName {
    repr: Repr,
}

enum Repr {
    Standard(StandardHeader),
    Custom(ByteStr),
}

struct StandardHeader {
    name: &'static str,
    hash: u16,
}

pub(crate) static PLACEHOLDER: HeaderName = HeaderName {
    repr: Repr::Standard(StandardHeader {
        name: "",
        hash: 0,
    })
};

impl HeaderName {
    /// Create new [`HeaderName`].
    pub fn new(name: impl Into<ByteStr>) -> Self {
        Self { repr: Repr::Custom(name.into()) }
    }

    pub(crate) const PLACEHOLDER: Self = Self {
        repr: Repr::Standard(StandardHeader {
            name: "",
            hash: 0,
        })
    };

    pub fn as_str(&self) -> &str {
        match &self.repr {
            Repr::Standard(s) => s.name,
            Repr::Custom(s) => s.as_str(),
        }
    }
}

pub(crate) trait Sealed: Sized {
    fn hash(&self) -> u16;

    fn as_str(&self) -> &str;

    fn as_header_name(&self) -> &HeaderName;
}

impl<S: Sealed> Sealed for &S {
    fn hash(&self) -> u16 {
        S::hash(self)
    }

    fn as_str(&self) -> &str {
        S::as_str(self)
    }

    fn as_header_name(&self) -> &HeaderName {
        S::as_header_name(self)
    }
}

impl Sealed for HeaderName {
    fn hash(&self) -> u16 {
        match &self.repr {
            Repr::Standard(s) => s.hash,
            Repr::Custom(s) => hash_str(s),
        }
    }

    fn as_str(&self) -> &str {
        HeaderName::as_str(self)
    }

    fn as_header_name(&self) -> &HeaderName {
        self
    }
}

impl Sealed for &'static str {
    fn hash(&self) -> u16 {
        hash_str(self)
    }

    fn as_str(&self) -> &str {
        self
    }

    fn as_header_name(&self) -> &HeaderName {
        &HeaderName { repr: Repr::Custom(ByteStr::from_static(self)) }
    }
}

fn hash_str(s: &str) -> u16 {
    let mut hasher = fnv::FnvHasher::with_key(199);
    hasher.write(s.as_bytes());
    hasher.finish() as _
}

// ===== Marker Trait =====

pub trait IntoHeaderName: Sealed { }
impl IntoHeaderName for HeaderName { }
impl IntoHeaderName for &'static str { }
impl<K: IntoHeaderName> IntoHeaderName for &K { }

// ===== Debug =====

impl std::fmt::Debug for HeaderName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeaderName").finish()
    }
}

