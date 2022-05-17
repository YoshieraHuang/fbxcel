use std::{io::{SeekFrom, Result}, pin::Pin, task::{Context, Poll}};

use futures_lite::{AsyncRead, AsyncSeek, AsyncSeekExt, AsyncBufRead};
use pin_project_lite::pin_project;

pin_project! {
    /// A reader with position cache.
    ///
    /// # Panics
    ///
    /// Panics if the position overflows.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub struct InnerAsyncPositionReader<R> {
        #[pin]
        inner: R,
        position: usize
    }
}

impl<R> InnerAsyncPositionReader<R> 
where
    R: AsyncRead + Unpin
{
    /// Create a new `SimpleAsyncPositionReader`
    pub fn new(inner: R) -> Self {
        Self { inner, position: 0 }
    }

    pub fn with_offset(inner: R, offset: usize) -> Self {
        Self { inner, position: offset }
    }

    pub fn into_inner(self) -> R {
        self.inner
    } 

    pub fn position(&self) -> usize {
        self.position
    }

    pub async fn skip_distance(&mut self, distance: u64) -> Result<()>
    where
        R: AsyncSeek
    {
        let mut distance = distance;
        while distance > 0 {
            let part = std::cmp::min(distance, i64::MAX as u64);
            self.inner.seek(SeekFrom::Current(part as i64)).await?;
            self.advance(part as usize);
            distance -= part;
        }
        Ok(())
    }

    #[inline]
    pub fn advance(&mut self, n: usize) {
        self.position += n;
    }
}

impl<R> AsyncRead for InnerAsyncPositionReader<R>
where
    R: AsyncRead + Unpin + Send
{
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }
    
    fn poll_read_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &mut [std::io::IoSliceMut<'_>]) -> Poll<Result<usize>> {
        self.project().inner.poll_read_vectored(cx, bufs)
    }
}

impl<R> AsyncBufRead for InnerAsyncPositionReader<R>
where
    R: AsyncBufRead + Unpin + Send
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.project().inner.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt)
    }
}
