//! Low-level data types for binary and stirng type node attributes.

use byte_order_reader::{ready_ok, FromAsyncReader, ReadU32};
use byteorder::LE;
use futures_util::{AsyncRead, Future};
use pin_project_lite::pin_project;
use std::{
    io::{Error, Result},
    pin::Pin,
    task::{Context, Poll},
};

/// A header type for array-type attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpecialAttributeHeader {
    /// Elements length in bytes.
    pub bytelen: u32,
}

impl<R> FromAsyncReader<R> for SpecialAttributeHeader
where
    R: AsyncRead + Unpin + Send,
{
    type Error = Error;
    type Fut<'a> = SpecialAttributeHeaderFut<'a, R> where R: 'a;

    fn from_async_reader(reader: &mut R) -> Self::Fut<'_> {
        SpecialAttributeHeaderFut::new(reader)
    }
}

pin_project! {
    pub struct SpecialAttributeHeaderFut<'a, R> {
        #[pin]
        inner: ReadU32<&'a mut R, LE>,
    }
}

impl<'a, R> SpecialAttributeHeaderFut<'a, R>
where
    R: AsyncRead + Unpin + Send + 'a,
{
    fn new(reader: &'a mut R) -> Self {
        let inner = u32::from_async_reader(reader);
        Self { inner }
    }
}

impl<'a, R> Future for SpecialAttributeHeaderFut<'a, R>
where
    R: AsyncRead + Unpin + Send + 'a,
{
    type Output = Result<SpecialAttributeHeader>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let bytelen = ready_ok!(self.project().inner.poll(cx));
        Poll::Ready(Ok(SpecialAttributeHeader { bytelen }))
    }
}
