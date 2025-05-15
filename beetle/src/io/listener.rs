use std::{
    io,
    net::SocketAddr,
    task::{Context, Poll},
};

pub trait Listener: Sized {
    type Stream;

    fn poll_accept(
        &self,
        cx: &mut Context,
    ) -> Poll<io::Result<(Self::Stream, impl Into<SocketAddr>)>>;
}

#[cfg(feature = "tokio")]
mod rt_tokio {
    use tokio::net::{TcpListener, TcpStream};

    use super::*;

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
