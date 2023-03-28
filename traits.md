
# Traits

```rs
// accept Vec<u8> or [u8]
fn end<B: AsRef<[u8]>>(buffer: B);

// accept String or &str
fn print<S: AsRef<str>>(s: S);

// accept String or &str
fn path<P: AsRef<Path>>(path: P)

```

## Todo

read about closures that can capture outer variable