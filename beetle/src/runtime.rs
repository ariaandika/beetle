//! entrypoint to start the server
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
};

use crate::{
    Service,
    io::Listener,
    net::Socket,
    service::{HttpService, tcp::TcpService},
};

#[cfg(feature = "tokio")]
pub use rt_tokio::{Tokio, TokioServe, listen};

pub fn serve<R: Runtime, S: HttpService>(listener: R::Listener, service: S) -> Serve<R, S> {
    Serve {
        listener,
        service: Arc::new(service),
    }
}

// ===== Runtime =====

pub trait Runtime {
    type Listener: Listener;

    fn spawn<F>(future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}

// ===== Futures =====

#[derive(Debug)]
pub struct Serve<R, S>
where
    R: Runtime,
{
    listener: R::Listener,
    service: Arc<S>,
}

impl<R, S> Future for Serve<R, S>
where
    R: Runtime,
    <R::Listener as Listener>::Stream: Into<Socket>,
    S: HttpService,
{
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match ready!(self.listener.poll_accept(cx)) {
                Ok((io, _)) => {
                    R::spawn(TcpService::new(self.service.clone()).call(io.into()));
                }
                Err(_err) => {}
            }
        }
    }
}

// ===== Tokio =====
#[cfg(feature = "tokio")]
mod rt_tokio {
    use std::hint;

    use tokio::net::{TcpListener, ToSocketAddrs};

    use super::*;

    /// [`Runtime`] implementation for [`tokio`].
    pub struct Tokio;

    impl Runtime for Tokio {
        type Listener = TcpListener;

        fn spawn<F>(future: F)
        where
            F: Future + Send + 'static,
            F::Output: Send + 'static,
        {
            tokio::spawn(future);
        }
    }

    /// Start the server using [`TcpListener`][tokio::net::TcpListener].
    ///
    /// This requires `tokio` features to be enabled.
    pub fn listen<A: ToSocketAddrs + 'static, S: HttpService>(
        addr: A,
        service: S,
    ) -> TokioServe<S> {
        TokioServe {
            phase: Phase::F1 { f: Box::pin(TcpListener::bind(addr)), s: service },
        }
    }

    pin_project_lite::pin_project! {
        pub struct TokioServe<S> {
            #[pin] phase: Phase<S>,
        }
    }

    pin_project_lite::pin_project! {
        #[project = Project]
        #[project_replace = Replace]
        enum Phase<S> {
            F1 { f: Pin<Box<dyn Future<Output = io::Result<TcpListener>>>>, s: S },
            F2 { #[pin] s: Serve<Tokio, S> },
            Deez,
        }
    }

    impl<S> Future for TokioServe<S>
    where
        S: HttpService,
    {
        type Output = io::Result<()>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let mut me = self.as_mut().project();

            match me.phase.as_mut().project() {
                Project::F1 { f, s: _ } => {
                    let io = ready!(f.as_mut().poll(cx)?);
                    let Replace::F1 { f: _, s } = me.phase.as_mut().project_replace(Phase::Deez) else {
                        unsafe { hint::unreachable_unchecked() }
                    };
                    me.phase.set(Phase::F2 { s: super::serve(io, s) });
                    self.poll(cx)
                },
                Project::F2 { s } => s.poll(cx),
                Project::Deez => unsafe { hint::unreachable_unchecked() },
            }
        }
    }
}

