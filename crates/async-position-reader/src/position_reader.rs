use std::{
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use futures_lite::{AsyncBufRead, AsyncRead, AsyncSeek};
use pin_project_lite::pin_project;

use crate::{AsyncPositionRead, InnerAsyncPositionReader};

pin_project! {
    /// Reader with seekable backend.
    /// Only works with reader implementing `AsyncSeek` trait.
    pub struct SeekableReader<R> {
        #[pin]
        inner: InnerAsyncPositionReader<R>
    }
}

impl<R> SeekableReader<R>
where
    R: AsyncRead + AsyncSeek + Unpin + Send,
{
    /// Create a new `SeekableReader`
    pub fn new(inner: R) -> Self {
        Self {
            inner: InnerAsyncPositionReader::new(inner),
        }
    }

    pub fn with_offset(inner: R, offset: usize) -> Self {
        Self {
            inner: InnerAsyncPositionReader::with_offset(inner, offset),
        }
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
        self.project().inner.poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [std::io::IoSliceMut<'_>],
    ) -> Poll<Result<usize>> {
        self.project().inner.poll_read_vectored(cx, bufs)
    }
}

impl<R> AsyncBufRead for SeekableReader<R>
where
    R: AsyncBufRead + Unpin + Send,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.project().inner.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt)
    }
}

#[async_trait]
impl<R> AsyncPositionRead for SeekableReader<R>
where
    R: AsyncRead + AsyncSeek + Unpin + Send,
{
    fn position(&self) -> u64 {
        self.inner.position() as u64
    }

    async fn skip_distance(&mut self, distance: u64) -> Result<()> {
        self.inner.skip_distance(distance).await
    }
}

pin_project! {
    /// Simple Reader that works with any async reader types
    pub struct SimpleReader<R> {
        #[pin]
        inner: InnerAsyncPositionReader<R>
    }
}

impl<R> SimpleReader<R>
where
    R: AsyncRead + Unpin + Send,
{
    pub fn new(inner: R) -> Self {
        Self {
            inner: InnerAsyncPositionReader::new(inner),
        }
    }

    pub fn with_offset(inner: R, offset: usize) -> Self {
        Self {
            inner: InnerAsyncPositionReader::with_offset(inner, offset),
        }
    }
}

impl<R> AsyncRead for SimpleReader<R>
where
    R: AsyncRead + Unpin + Send,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        self.project().inner.poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [std::io::IoSliceMut<'_>],
    ) -> Poll<Result<usize>> {
        self.project().inner.poll_read_vectored(cx, bufs)
    }
}

impl<R> AsyncBufRead for SimpleReader<R>
where
    R: AsyncBufRead + Unpin + Send,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.project().inner.poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.project().inner.consume(amt)
    }
}

impl<R> AsyncPositionRead for SimpleReader<R>
where
    R: AsyncRead + Unpin + Send,
{
    fn position(&self) -> u64 {
        self.inner.position() as u64
    }
}
