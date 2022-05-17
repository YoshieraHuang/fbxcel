//! Low-level data types for binary and stirng type node attributes.

use async_trait::async_trait;
use futures_lite::AsyncRead;
use byte_order_reader::FromAsyncReader;

use crate::pull_parser::Error as ParserError;

/// A header type for array-type attributes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SpecialAttributeHeader {
    /// Elements length in bytes.
    pub(crate) bytelen: u32,
}

#[async_trait]
impl<R> FromAsyncReader for SpecialAttributeHeader
where
    R: AsyncRead + Unpin + Send,
{
    type Error = ParserError;

    async fn from_async_reader(reader: &mut R) -> Result<Self, ParserError> {
        u32::from_async_reader(reader).map(|res| res.map(|bytelen| Self { bytelen }))
    }
}
