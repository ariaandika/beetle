//! entrypoint to start the server
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, ready},
};
use tokio::net::{TcpListener, ToSocketAddrs};

use crate::service::{HttpService, Service, tcp::TcpService};

pub fn listen<S: HttpService>(
    addr: impl ToSocketAddrs,
    service: S,
) -> Serve<S, impl Future<Output = io::Result<TcpListener>>> {
    Serve {
        phase: Phase::P1 {
            f: TcpListener::bind(addr),
        },
        service: Arc::new(service),
    }
}

pin_project_lite::pin_project! {
    pub struct Serve<S, F> {
        #[pin]
        phase: Phase<F>,
        service: Arc<S>,
    }
}

pin_project_lite::pin_project! {
    #[project = PJ]
    enum Phase<F> {
        P1 { #[pin] f: F },
        P2 { tcp: TcpListener },
    }
}

impl<S, F> Future for Serve<S, F>
where
    S: HttpService,
    F: Future<Output = io::Result<TcpListener>>,
{
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut me = self.project();

        if let PJ::P1 { f } = me.phase.as_mut().project() {
            let ok = ready!(f.poll(cx)?);
            me.phase.set(Phase::P2 { tcp: ok });
        }

        let PJ::P2 { tcp } = me.phase.as_mut().project() else {
            unreachable!()
        };

        loop {
            match ready!(tcp.poll_accept(cx)) {
                Ok((io, _)) => {
                    tokio::spawn(TcpService::new(me.service.clone()).call(io));
                }
                Err(_err) => {}
            }
        }
    }
}

