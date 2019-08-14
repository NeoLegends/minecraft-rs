macro_rules! derive_async_read {
    ($stream:ident) => {
        #[inline(always)]
        unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
            self.$stream.prepare_uninitialized_buffer(buf)
        }

        #[inline(always)]
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<Result<usize>> {
            Pin::new(&mut self.get_mut().$stream).poll_read(cx, buf)
        }
    };
}

macro_rules! derive_async_write {
    ($stream:ident) => {
        #[inline(always)]
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            Pin::new(&mut self.get_mut().$stream).poll_write(cx, buf)
        }

        #[inline(always)]
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            Pin::new(&mut self.get_mut().$stream).poll_flush(cx)
        }

        #[inline(always)]
        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<()>> {
            Pin::new(&mut self.get_mut().$stream).poll_shutdown(cx)
        }
    };
}
