macro_rules! derive_async_read {
    ($stream:ident) => {
        #[inline]
        unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
            self.$stream.prepare_uninitialized_buffer(buf)
        }

        #[inline]
        fn poll_read(
            self: ::std::pin::Pin<&mut Self>,
            cx: &mut ::std::task::Context<'_>,
            buf: &mut [u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
            ::std::pin::Pin::new(&mut self.get_mut().$stream).poll_read(cx, buf)
        }
    };
}

macro_rules! derive_async_write {
    ($stream:ident) => {
        #[inline]
        fn poll_write(
            self: ::std::pin::Pin<&mut Self>,
            cx: &mut ::std::task::Context<'_>,
            buf: &[u8],
        ) -> ::std::task::Poll<::std::io::Result<usize>> {
            ::std::pin::Pin::new(&mut self.get_mut().$stream).poll_write(cx, buf)
        }

        #[inline]
        fn poll_flush(
            self: ::std::pin::Pin<&mut Self>,
            cx: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
            ::std::pin::Pin::new(&mut self.get_mut().$stream).poll_flush(cx)
        }

        #[inline]
        fn poll_shutdown(
            self: ::std::pin::Pin<&mut Self>,
            cx: &mut ::std::task::Context<'_>,
        ) -> ::std::task::Poll<::std::io::Result<()>> {
            ::std::pin::Pin::new(&mut self.get_mut().$stream).poll_shutdown(cx)
        }
    };
}
