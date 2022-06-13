//! Low-level data types related to array type node attributes.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use byte_order_reader::{ready_ok, FromAsyncReader, ReadU32};
use byteorder::LE;
use futures_util::{future::BoxFuture, AsyncRead, Future};
use pin_project_lite::pin_project;

use crate::error::LowError;

/// Array attribute encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrayAttributeEncoding {
    /// Direct values.
    Direct,
    /// Zlib compression.
    ///
    /// Zlib compression with header.
    Zlib,
}

impl ArrayAttributeEncoding {
    /// Creates a new `ArrayEncoding` from the given raw value.
    pub fn from_u32(v: u32) -> Result<Self, LowError> {
        match v {
            0 => Ok(ArrayAttributeEncoding::Direct),
            1 => Ok(ArrayAttributeEncoding::Zlib),
            v => Err(LowError::InvalidArrayAttributeEncoding(v)),
        }
    }

    /// Returns the raw value.
    #[cfg(feature = "writer")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
    pub fn to_u32(self) -> u32 {
        match self {
            ArrayAttributeEncoding::Direct => 0,
            ArrayAttributeEncoding::Zlib => 1,
        }
    }
}

impl<R> FromAsyncReader<R> for ArrayAttributeEncoding
where
    R: AsyncRead + Unpin + Send,
{
    type Error = LowError;
    type Fut<'a> = ArrayAttributeEncodingFut<'a, R> where R: 'a;

    fn from_async_reader(reader: &mut R) -> Self::Fut<'_> {
        Self::Fut::new(reader)
    }
}

pin_project! {
    pub struct ArrayAttributeEncodingFut<'a, R> {
        #[pin]
        inner: ReadU32<&'a mut R, LE>,
    }
}

impl<'a, R> ArrayAttributeEncodingFut<'a, R>
where
    R: AsyncRead + Unpin + Send + 'a,
{
    fn new(reader: &'a mut R) -> Self {
        let inner = u32::from_async_reader(reader);
        Self { inner }
    }
}

impl<'a, R> Future for ArrayAttributeEncodingFut<'a, R>
where
    R: AsyncRead + Unpin + Send + 'a,
{
    type Output = Result<ArrayAttributeEncoding, LowError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let raw_encoding = ready_ok!(self.project().inner.poll(cx));
        Poll::Ready(ArrayAttributeEncoding::from_u32(raw_encoding))
    }
}

/// A header type for array-type attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArrayAttributeHeader {
    /// Number of elements.
    pub elements_count: u32,
    /// Encoding.
    pub encoding: ArrayAttributeEncoding,
    /// Elements length in bytes.
    pub bytelen: u32,
}

impl<R> FromAsyncReader<R> for ArrayAttributeHeader
where
    R: AsyncRead + Unpin + Send,
{
    type Error = LowError;
    type Fut<'a> = BoxFuture<'a, Result<Self, Self::Error>> where R: 'a;

    fn from_async_reader(reader: &mut R) -> Self::Fut<'_> {
        Box::pin(async move {
            let elements_count = u32::from_async_reader(reader).await?;
            let encoding = ArrayAttributeEncoding::from_async_reader(reader).await?;
            let bytelen = u32::from_async_reader(reader).await?;

            Ok(Self {
                elements_count,
                encoding,
                bytelen,
            })
        })
    }
}
