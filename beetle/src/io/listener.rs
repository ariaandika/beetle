use std::{
    io,
    net::SocketAddr,
    task::{Context, Poll},
};

pub trait BoundListener<Addr> {
    type Listener: Listener;

    type Future: Future<Output = io::Result<Self::Listener>>;

    fn bind(addr: Addr) -> Self::Future;
}

pub trait Listener: Sized {
    type Stream;

    fn poll_accept(
        &self,
        cx: &mut Context,
    ) -> Poll<io::Result<(Self::Stream, impl Into<SocketAddr>)>>;
}

#[cfg(feature = "tokio")]
mod rt_tokio {
    use std::pin::Pin;
    use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

    use super::*;

    impl<Addr: ToSocketAddrs + 'static> BoundListener<Addr> for TcpListener {
        type Listener = Self;

        // #63063 <https://github.com/rust-lang/rust/issues/63063>
        type Future = Pin<Box<dyn Future<Output = io::Result<TcpListener>>>>;

        fn bind(addr: Addr) -> Self::Future {
            Box::pin(Self::bind(addr))
        }
    }

    impl Listener for TcpListener {
        type Stream = TcpStream;

        fn poll_accept(
            &self,
            cx: &mut Context,
        ) -> Poll<io::Result<(Self::Stream, impl Into<SocketAddr>)>> {
            Self::poll_accept(self, cx)
        }
    }
}
