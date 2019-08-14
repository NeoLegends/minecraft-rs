use futures::ready;
use std::{
    io::Result,
    marker::Unpin,
    mem,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Clone, Debug)]
pub struct Autoflush<S> {
    stream: S,
    written_not_flushed: usize,
}

impl<S> Autoflush<S> {
    pub fn new(stream: S) -> Self {
        Autoflush {
            stream,
            written_not_flushed: 0,
        }
    }

    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    pub fn into_inner(self) -> S {
        self.stream
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for Autoflush<S> {
    derive_async_read!(stream);
}

impl<S: AsyncWrite + Unpin> AsyncWrite for Autoflush<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        let pinned = self.get_mut();
        let mut stream = Pin::new(&mut pinned.stream);

        let mut nread = ready!(stream.as_mut().poll_write(cx, buf))?;

        match stream.as_mut().poll_flush(cx) {
            Poll::Pending => {
                pinned.written_not_flushed += nread;
                return Poll::Pending;
            }
            Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
            Poll::Ready(Ok(_)) => {
                nread += mem::replace(&mut pinned.written_not_flushed, 0);
            }
        }

        Poll::Ready(Ok(nread))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}
