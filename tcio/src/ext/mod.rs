
/// Helper trait to [`Display`][std::fmt::Display] bytes.
pub trait FmtExt {
    /// Lossy [`Display`][std::fmt::Display] bytes.
    fn lossy(&self) -> LossyFmt<'_>;
}

/// Lossy [`Display`][std::fmt::Display] implementation for bytes.
pub struct LossyFmt<'a>(pub &'a [u8]);

impl FmtExt for [u8] {
    fn lossy(&self) -> LossyFmt<'_> {
        LossyFmt(self)
    }
}

impl std::fmt::Display for LossyFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &b in self.0 {
            if b.is_ascii_graphic() || b.is_ascii_whitespace() {
                write!(f, "{}", b as char)?;
            } else {
                write!(f, "\\x{b:x}")?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for LossyFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "b\"{self}\"")
    }
}

