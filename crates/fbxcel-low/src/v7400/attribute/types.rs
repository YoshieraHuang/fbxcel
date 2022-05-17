//! Node attribute type.

use crate::error::LowError;
use async_trait::async_trait;
use byte_order_reader::{AsyncByteOrderRead, FromAsyncReader};
use futures_lite::AsyncRead;

/// Node attribute type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeType {
    /// Single `bool`.
    Bool,
    /// Single `i16`.
    I16,
    /// Single `i32`.
    I32,
    /// Single `i64`.
    I64,
    /// Single `f32`.
    F32,
    /// Single `f64`.
    F64,
    /// Array of `bool`.
    ArrBool,
    /// Array of `i32`.
    ArrI32,
    /// Array of `i64`.
    ArrI64,
    /// Array of `f32`.
    ArrF32,
    /// Array of `f64`.
    ArrF64,
    /// Binary.
    Binary,
    /// UTF-8 string.
    String,
}

impl AttributeType {
    /// Creates an `AttributeType` from the given type code.
    pub(crate) fn from_type_code(code: u8) -> Result<Self, LowError> {
        match code {
            b'C' => Ok(AttributeType::Bool),
            b'Y' => Ok(AttributeType::I16),
            b'I' => Ok(AttributeType::I32),
            b'L' => Ok(AttributeType::I64),
            b'F' => Ok(AttributeType::F32),
            b'D' => Ok(AttributeType::F64),
            b'b' => Ok(AttributeType::ArrBool),
            b'i' => Ok(AttributeType::ArrI32),
            b'l' => Ok(AttributeType::ArrI64),
            b'f' => Ok(AttributeType::ArrF32),
            b'd' => Ok(AttributeType::ArrF64),
            b'R' => Ok(AttributeType::Binary),
            b'S' => Ok(AttributeType::String),
            code => Err(LowError::InvalidAttributeTypeCode(code)),
        }
    }

    /// Returns the type code.
    #[cfg(feature = "writer")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
    pub(crate) fn type_code(self) -> u8 {
        match self {
            AttributeType::Bool => b'C',
            AttributeType::I16 => b'Y',
            AttributeType::I32 => b'I',
            AttributeType::I64 => b'L',
            AttributeType::F32 => b'F',
            AttributeType::F64 => b'D',
            AttributeType::ArrBool => b'b',
            AttributeType::ArrI32 => b'i',
            AttributeType::ArrI64 => b'l',
            AttributeType::ArrF32 => b'f',
            AttributeType::ArrF64 => b'd',
            AttributeType::Binary => b'R',
            AttributeType::String => b'S',
        }
    }
}

#[async_trait]
impl<R> FromAsyncReader<R> for AttributeType
where
    R: AsyncRead + Unpin + Send,
{
    type Error = LowError;

    async fn from_async_reader(reader: &mut R) -> Result<Self, LowError> {
        let type_code = reader.read_u8().await?;
        let attr_type = Self::from_type_code(type_code)?;
        Ok(attr_type)
    }
}
