use std::{
    io,
    net::SocketAddr,
    task::{Context, Poll},
};

pub trait Runtime<Addr> {
    type Listener: Listener;

    type BindFuture: Future<Output = io::Result<Self::Listener>>;

    fn bind(addr: Addr) -> Self::BindFuture;

    fn spawn<F>(future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
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

    impl<Addr: ToSocketAddrs + 'static> Runtime<Addr> for TcpListener {
        type Listener = Self;

        // #63063 <https://github.com/rust-lang/rust/issues/63063>
        type BindFuture = Pin<Box<dyn Future<Output = io::Result<TcpListener>>>>;

        fn bind(addr: Addr) -> Self::BindFuture {
            Box::pin(Self::bind(addr))
        }

        fn spawn<F>(future: F)
        where
            F: Future + Send + 'static,
            F::Output: Send + 'static,
        {
            tokio::spawn(future);
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
