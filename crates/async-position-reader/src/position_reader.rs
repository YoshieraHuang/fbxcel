use std::{
    io::{Result, SeekFrom},
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures_util::{ready, AsyncBufRead, AsyncRead, AsyncSeek, AsyncSeekExt};
use pin_project_lite::pin_project;

use crate::AsyncPositionRead;

pin_project! {
    /// Reader with seekable backend.
    /// Only works with reader implementing `AsyncSeek` trait.
    #[derive(Debug)]
    pub struct SeekableReader<R> {
        #[pin]
        inner: R,
        // cached position
        position: u64,
    }
}

impl<R> SeekableReader<R>
where
    R: AsyncRead + AsyncSeek + Unpin + Send,
{
    /// Create a new `SeekableReader`
    pub fn new(inner: R) -> Self {
        Self { inner, position: 0 }
    }

    pub fn with_offset(inner: R, offset: u64) -> Self {
        Self {
            inner,
            position: offset,
        }
    }

    pub fn advance(&mut self, offset: u64) {
        self.position += offset;
    }
}

impl<R> AsyncRead for SeekableReader<R>
where
    R: AsyncRead + Unpin + Send,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        let this = self.project();
        let n = ready!(this.inner.poll_read(cx, buf));
        if let Ok(n) = n {
            *(this.position) += n as u64;
        }
        Poll::Ready(n)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [std::io::IoSliceMut<'_>],
    ) -> Poll<Result<usize>> {
        let this = self.project();
        let n = ready!(this.inner.poll_read_vectored(cx, bufs));
        if let Ok(n) = n {
            *(this.position) += n as u64;
        }
        Poll::Ready(n)
    }
}

impl<R> AsyncSeek for SeekableReader<R>
where
    R: AsyncSeek + Unpin + Send,
{
    fn poll_seek(self: Pin<&mut Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<Result<u64>> {
        self.project().inner.poll_seek(cx, pos)
    }
}

impl<R> AsyncBufRead for SeekableReader<R>
where
    R: AsyncBufRead + AsyncSeek + Unpin + Send,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.project().inner.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let this = self.project();
        this.inner.consume(amt);
        *(this.position) += amt as u64;
    }
}

#[async_trait]
impl<R> AsyncPositionRead for SeekableReader<R>
where
    R: AsyncRead + AsyncSeek + Unpin + Send,
{
    fn position(&self) -> u64 {
        self.position
    }

    async fn skip_distance(&mut self, mut distance: u64) -> Result<()> {
        while distance > 0 {
            let part = std::cmp::min(distance, std::i64::MAX as u64);
            self.inner.seek(SeekFrom::Current(part as i64)).await?;
            self.advance(part);
            distance -= part;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{io::Cursor, AsyncRead, AsyncReadExt};

    fn prepare_iota() -> Cursor<Vec<u8>> {
        let orig = (0..=255).collect::<Vec<u8>>();
        Cursor::new(orig)
    }

    #[async_std::test]
    async fn read() {
        let reader = SeekableReader::new(prepare_iota());
        assert_eq!(
            reader.position(),
            0,
            "`PositionCacheReader::new()` should return a reader with position 0"
        );
        check_read_with_offset(reader, 0).await;
    }

    #[async_std::test]
    async fn read_with_offset() -> anyhow::Result<()> {
        const OFFSET: u64 = 60;
        let reader = SeekableReader::with_offset(prepare_iota(), OFFSET);
        assert_eq!(
            reader.position(),
            OFFSET,
            "`PositionCacheReader::with_offset()` should return a reader with the given offset"
        );
        check_read_with_offset(reader, OFFSET).await;
        Ok(())
    }

    async fn check_read_with_offset<R: AsyncRead + AsyncSeek + Unpin + Send>(
        mut reader: SeekableReader<R>,
        offset: u64,
    ) {
        const BUF_SIZE: usize = 128;

        let mut buf = [0; BUF_SIZE];
        let size = reader
            .read(&mut buf)
            .await
            .expect("Read from `Cursor<Vec<u8>>` should never fail");

        assert!(
            size > 0,
            "Read from non-empty `Cursor<Vec<u8>>` should obtain some data"
        );
        // "Offset" is for internal count, not for content to be read.
        // So here use `0..size`, not `OFFSET..(OFFSET+size)`.
        assert_eq!(
            &buf[..size],
            &(0..size as u8).into_iter().collect::<Vec<u8>>()[..],
            "Read should obtain correct data"
        );
        assert_eq!(
            reader.position() as usize,
            offset as usize + size,
            "Position should be correctly updated after a read"
        );
    }
}
