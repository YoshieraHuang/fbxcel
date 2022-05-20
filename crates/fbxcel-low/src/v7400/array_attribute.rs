//! Low-level data types related to array type node attributes.

use async_trait::async_trait;
use byte_order_reader::FromAsyncReader;
use futures_lite::AsyncRead;

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

#[async_trait]
impl<R> FromAsyncReader<R> for ArrayAttributeEncoding
where
    R: AsyncRead + Unpin + Send,
{
    type Error = LowError;

    async fn from_async_reader(reader: &mut R) -> Result<Self, Self::Error> {
        let raw_encoding = u32::from_async_reader(reader).await?;
        ArrayAttributeEncoding::from_u32(raw_encoding)
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

#[async_trait]
impl<R> FromAsyncReader<R> for ArrayAttributeHeader
where
    R: AsyncRead + Unpin + Send,
{
    type Error = LowError;

    async fn from_async_reader(reader: &mut R) -> Result<Self, LowError> {
        let elements_count = u32::from_async_reader(reader).await?;
        let encoding = ArrayAttributeEncoding::from_async_reader(reader).await?;
        let bytelen = u32::from_async_reader(reader).await?;

        Ok(Self {
            elements_count,
            encoding,
            bytelen,
        })
    }
}
