//! Low-level data types for binary and stirng type node attributes.

use async_trait::async_trait;
use byte_order_reader::FromAsyncReader;
use futures_lite::AsyncRead;
use std::io::{Error, Result};

/// A header type for array-type attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpecialAttributeHeader {
    /// Elements length in bytes.
    pub bytelen: u32,
}

#[async_trait]
impl<R> FromAsyncReader<R> for SpecialAttributeHeader
where
    R: AsyncRead + Unpin + Send,
{
    type Error = Error;

    async fn from_async_reader(reader: &mut R) -> Result<Self> {
        let bytelen = u32::from_async_reader(reader).await?;
        Ok(SpecialAttributeHeader { bytelen })
    }
}
