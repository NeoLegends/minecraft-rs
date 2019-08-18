use futures::ready;
use openssl::symm::{Cipher, Crypter, Mode};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    io::{Error, ErrorKind, Read, Result},
    marker::Unpin,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};

const BUF_SIZE: usize = 1024 * 8;

#[derive(Debug)]
pub struct CryptStream<S> {
    stream: Encrypt<Decrypt<S>>,
}

struct Decrypt<S> {
    decrypt: Crypter,
    is_eof: bool,
    stream: S,

    de_buf: Box<[u8]>,
    de_buf_pos: usize,
    de_buf_len: usize,

    r_buf: Box<[u8]>,
    r_buf_len: usize,
}

struct Encrypt<S> {
    encrypt: Crypter,
    stream: S,

    buf: Box<[u8]>,
    buf_pos: usize,
    buf_len: usize,
}

impl<S> CryptStream<S> {
    pub fn new(stream: S, key: &[u8], iv: &[u8]) -> Self {
        Self::with_capacity(stream, key, iv, BUF_SIZE)
    }

    pub fn with_capacity(stream: S, key: &[u8], iv: &[u8], cap: usize) -> Self {
        let decrypt = Decrypt::with_capacity(stream, key, iv, cap);
        let encrypt = Encrypt::with_capacity(decrypt, key, iv, cap);

        CryptStream { stream: encrypt }
    }

    pub fn get_ref(&self) -> &S {
        self.stream.get_ref().get_ref()
    }

    pub fn get_mut(&mut self) -> &mut S {
        self.stream.get_mut().get_mut()
    }

    pub fn into_inner(self) -> S {
        self.stream.into_inner().into_inner()
    }
}

impl<S: AsyncRead + Unpin> AsyncRead for CryptStream<S> {
    derive_async_read!(stream);
}

impl<S: AsyncWrite + Unpin> AsyncWrite for CryptStream<S> {
    derive_async_write!(stream);
}

impl<S> Decrypt<S> {
    pub fn new(stream: S, key: &[u8], iv: &[u8]) -> Self {
        Self::with_capacity(stream, key, iv, BUF_SIZE)
    }

    pub fn with_capacity(stream: S, key: &[u8], iv: &[u8], cap: usize) -> Self {
        assert!(cap > 0);

        let cipher = Cipher::aes_128_cfb8();
        let decrypt = Crypter::new(cipher, Mode::Decrypt, key, Some(iv))
            .expect("failed to set up decrypter");

        Decrypt {
            decrypt,
            is_eof: false,
            stream,

            de_buf: vec![0; cap].into_boxed_slice(),
            de_buf_pos: 0,
            de_buf_len: 0,

            r_buf: vec![0; cap].into_boxed_slice(),
            r_buf_len: 0,
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

impl<S: AsyncRead + Unpin> AsyncRead for Decrypt<S> {
    unsafe fn prepare_uninitialized_buffer(&self, _: &mut [u8]) -> bool {
        false
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let pinned = self.get_mut();
        let mut nread = 0;

        while !buf.is_empty() {
            if pinned.de_buf_len > 0 {
                let to = pinned.de_buf_pos + pinned.de_buf_len;
                let read = (&pinned.de_buf[pinned.de_buf_pos..to]).read(buf)?;

                assert!(read <= buf.len(), "read more bytes than possible");

                pinned.de_buf_pos += read;
                pinned.de_buf_len -= read;

                if pinned.de_buf_len == 0 {
                    pinned.de_buf_pos = 0;
                }

                nread += read;
                buf = &mut buf[read..];
            } else if pinned.r_buf_len > 0 {
                let decrypted = pinned
                    .decrypt
                    .update(&pinned.r_buf[..pinned.r_buf_len], &mut pinned.de_buf)
                    .map_err(|ssl_err| Error::new(ErrorKind::Other, ssl_err))?;

                pinned.de_buf_len += decrypted;
                pinned.r_buf_len = 0;
            } else if pinned.is_eof {
                if nread > 0 {
                    break;
                } else {
                    return Poll::Ready(Ok(0));
                }
            } else {
                let mut stream = Pin::new(&mut pinned.stream);
                unsafe {
                    stream
                        .as_mut()
                        .prepare_uninitialized_buffer(&mut pinned.r_buf);
                }

                let nread = match stream.poll_read(cx, &mut pinned.r_buf) {
                    Poll::Pending => {
                        if nread > 0 {
                            break;
                        } else {
                            return Poll::Pending;
                        }
                    }
                    Poll::Ready(res) => res?,
                };

                pinned.r_buf_len += nread;

                if nread == 0 {
                    let decrypted = pinned
                        .decrypt
                        .finalize(&mut pinned.de_buf)
                        .map_err(|ssl_err| Error::new(ErrorKind::Other, ssl_err))?;

                    pinned.de_buf_len += decrypted;
                    pinned.is_eof = true;
                }
            }
        }

        Poll::Ready(Ok(nread))
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for Decrypt<S> {
    derive_async_write!(stream);
}

impl<S: Debug> Debug for Decrypt<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("Decrypt")
            .field("decrypt", &"...")
            .field("de_buf", &self.de_buf)
            .field("de_buf_len", &self.de_buf_len)
            .field("de_buf_pos", &self.de_buf_pos)
            .field("r_buf", &self.r_buf)
            .field("r_buf_len", &self.r_buf_len)
            .field("stream", &self.stream)
            .finish()
    }
}

impl<S> Encrypt<S> {
    pub fn new(stream: S, key: &[u8], iv: &[u8]) -> Self {
        Self::with_capacity(stream, key, iv, BUF_SIZE)
    }

    pub fn with_capacity(stream: S, key: &[u8], iv: &[u8], cap: usize) -> Self {
        assert!(cap > 0);

        let encrypt =
            Crypter::new(Cipher::aes_128_cfb8(), Mode::Encrypt, key, Some(iv))
                .expect("failed to set up decrypter");

        Encrypt {
            encrypt,
            stream,

            buf: vec![0; cap].into_boxed_slice(),
            buf_pos: 0,
            buf_len: 0,
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

impl<S: AsyncRead + Unpin> AsyncRead for Encrypt<S> {
    derive_async_read!(stream);
}

impl<S: AsyncWrite + Unpin> AsyncWrite for Encrypt<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: &[u8],
    ) -> Poll<Result<usize>> {
        let mut nwritten = 0;

        while !buf.is_empty() {
            if (self.buf_pos + self.buf_len) == self.buf.len() {
                match self.as_mut().poll_flush(cx) {
                    Poll::Pending => {
                        if nwritten > 0 {
                            break;
                        } else {
                            return Poll::Pending;
                        }
                    }
                    Poll::Ready(res) => res?,
                }
            } else {
                let pinned = self.as_mut().get_mut();

                let remaining_in_buf =
                    pinned.buf.len() - (pinned.buf_pos + pinned.buf_len);
                let remaining = remaining_in_buf.min(buf.len());
                let write_to = &mut pinned.buf[(pinned.buf_pos + pinned.buf_len)..];

                let encrypted =
                    pinned
                        .encrypt
                        .update(&buf[..remaining], write_to)
                        .map_err(|ssl_err| Error::new(ErrorKind::Other, ssl_err))?;

                nwritten += remaining;
                pinned.buf_len += encrypted;

                buf = &buf[remaining..];
            }
        }

        Poll::Ready(Ok(nwritten))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let pinned = self.get_mut();
        let mut stream = Pin::new(&mut pinned.stream);

        while pinned.buf_len > 0 {
            let to_write =
                &pinned.buf[pinned.buf_pos..(pinned.buf_pos + pinned.buf_len)];
            let nwritten = ready!(stream.as_mut().poll_write(cx, to_write))?;

            pinned.buf_pos += nwritten;
            pinned.buf_len -= nwritten;
        }

        pinned.buf_pos = 0;
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<()>> {
        ready!(self.as_mut().poll_flush(cx))?;

        let pinned = self.get_mut();
        Pin::new(&mut pinned.stream).poll_shutdown(cx)
    }
}

impl<S: Debug> Debug for Encrypt<S> {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("Encrypt")
            .field("buf", &self.buf)
            .field("buf_len", &self.buf_len)
            .field("buf_pos", &self.buf_pos)
            .field("encrypt", &"...")
            .field("stream", &self.stream)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read, Write};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    struct Wrapper(pub Cursor<Vec<u8>>);

    impl Wrapper {
        pub fn new() -> Self {
            Self::from_buf(Vec::new())
        }

        pub fn from_buf(buf: Vec<u8>) -> Self {
            Wrapper(Cursor::new(buf))
        }
    }

    impl AsyncRead for Wrapper {
        fn poll_read(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<Result<usize>> {
            Poll::Ready(Read::read(&mut self.get_mut().0, buf))
        }
    }

    impl AsyncWrite for Wrapper {
        fn poll_write(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            Poll::Ready(self.get_mut().0.write(buf))
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    const TEST_STR: &[u8] =
        b"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, \
         sed diam nonumy eirmod tempor invidunt ut labore et dolore \
         magna aliquyam erat, sed diam voluptua. At vero eos et \
         accusam et justo duo dolores et ea rebum. Stet clita kasd \
         gubergren, no sea takimata sanctus est Lorem ipsum dolor \
         sit amet.";
    const TEST_STR_ENC: &[u8] = &[
        42, 234, 59, 238, 208, 211, 139, 226, 141, 36, 36, 104, 2, 118, 90, 0, 35,
        35, 11, 93, 238, 43, 191, 242, 28, 52, 165, 148, 186, 29, 109, 79, 151, 100,
        193, 54, 90, 227, 38, 50, 196, 145, 170, 219, 151, 131, 14, 197, 209, 211,
        53, 174, 205, 181, 53, 63, 179, 250, 179, 202, 53, 107, 160, 113, 126, 115,
        101, 66, 133, 172, 203, 224, 64, 62, 156, 151, 50, 16, 122, 214, 197, 10,
        230, 163, 86, 46, 154, 67, 156, 245, 32, 123, 194, 28, 21, 8, 110, 254, 1,
        18, 189, 37, 23, 15, 186, 137, 134, 215, 7, 58, 215, 47, 135, 134, 17, 26,
        22, 251, 3, 69, 35, 50, 167, 185, 149, 226, 246, 113, 21, 124, 72, 147, 227,
        100, 144, 250, 74, 107, 3, 85, 193, 173, 7, 17, 243, 18, 83, 5, 135, 104,
        204, 47, 144, 210, 141, 44, 2, 222, 185, 83, 1, 23, 25, 138, 198, 254, 126,
        31, 216, 140, 14, 231, 223, 199, 170, 3, 196, 40, 125, 232, 247, 36, 187,
        161, 139, 54, 109, 44, 119, 224, 68, 70, 167, 91, 21, 118, 90, 83, 191, 20,
        69, 163, 59, 103, 124, 108, 82, 160, 84, 100, 31, 185, 159, 244, 156, 79, 1,
        104, 188, 237, 228, 95, 235, 10, 143, 213, 97, 236, 77, 153, 221, 248, 143,
        198, 16, 132, 143, 241, 103, 178, 196, 123, 67, 31, 5, 54, 219, 205, 198,
        52, 114, 50, 145, 73, 131, 130, 28, 180, 198, 161, 182, 97, 38, 248, 145,
        91, 71, 101, 157, 125, 41, 65, 223, 39, 30, 107, 173, 153, 191, 250, 155,
        124, 0, 174, 39, 78, 220, 192, 188, 161, 21, 21, 177, 178, 234,
    ];

    #[tokio::test]
    async fn encrypt() {
        let mut inner = Wrapper::new();
        let mut stream = Encrypt::new(&mut inner, &vec![0; 16], &vec![0; 16]);

        stream.write(TEST_STR).await.unwrap();
        stream.shutdown().await.unwrap();

        assert_eq!(inner.0.into_inner(), TEST_STR_ENC);
    }

    #[tokio::test]
    async fn decrypt() {
        let mut inner = Wrapper::from_buf(TEST_STR_ENC.to_owned());
        let mut stream = Decrypt::new(&mut inner, &vec![0; 16], &vec![0; 16]);

        let mut r_buf = Vec::new();
        stream.read_to_end(&mut r_buf).await.unwrap();

        assert_eq!(r_buf, TEST_STR);
    }
}
