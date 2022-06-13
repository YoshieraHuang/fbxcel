//! Node attribute type.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::error::LowError;
use byte_order_reader::{ready_ok, AsyncByteOrderRead, FromAsyncReader, ReadU8};
use futures_util::{AsyncRead, Future};
use pin_project_lite::pin_project;

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
    pub fn from_type_code(code: u8) -> Result<Self, LowError> {
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
    pub fn type_code(self) -> u8 {
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

impl<R> FromAsyncReader<R> for AttributeType
where
    R: AsyncRead + Unpin + Send,
{
    type Error = LowError;
    type Fut<'a> = AttributeTypeFut<'a, R> where R: 'a;

    fn from_async_reader(reader: &mut R) -> Self::Fut<'_> {
        Self::Fut::new(reader)
    }
}

pin_project! {
    pub struct AttributeTypeFut<'a, R> {
        #[pin]
        inner: ReadU8<&'a mut R>,
    }
}

impl<'a, R> AttributeTypeFut<'a, R>
where
    R: AsyncRead + Unpin + Send + 'a,
{
    fn new(reader: &'a mut R) -> Self {
        let inner = reader.read_u8();
        Self { inner }
    }
}

impl<'a, R> Future for AttributeTypeFut<'a, R>
where
    R: AsyncRead + Unpin + Send + 'a,
{
    type Output = Result<AttributeType, LowError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let type_code = ready_ok!(self.project().inner.poll(cx));
        Poll::Ready(AttributeType::from_type_code(type_code))
    }
}
