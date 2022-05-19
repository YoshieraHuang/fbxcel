//! Array-type node attributes.

use std::{io, marker::PhantomData, pin::Pin, task::{Context, Poll}};

use byte_order_reader::{AsyncByteOrderRead, FromAsyncReader};
use futures_core::Stream;
use futures_lite::{AsyncRead, AsyncBufRead, ready, FutureExt};
use async_compression::futures::bufread::ZlibDecoder;

use fbxcel_low::v7400::ArrayAttributeEncoding;

use crate::pull_parser::Result;

/// Attribute stream decoder.
// `io::BufRead` is not implemented for `ZlibDecoder`.
#[derive(Debug)]
pub(crate) enum AttributeStreamDecoder<R> {
    /// Direct stream.
    Direct(R),
    /// Zlib-decoded stream.
    Zlib(ZlibDecoder<R>),
}

impl<R: AsyncBufRead> AttributeStreamDecoder<R> {
    /// Creates a new decoded reader.
    pub(crate) fn create(encoding: ArrayAttributeEncoding, reader: R) -> Self {
        match encoding {
            ArrayAttributeEncoding::Direct => AttributeStreamDecoder::Direct(reader),
            ArrayAttributeEncoding::Zlib => AttributeStreamDecoder::Zlib(
                ZlibDecoder::new(reader)
            ),
        }
    }
}

impl<R: AsyncBufRead + Unpin> AsyncRead for AttributeStreamDecoder<R> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        match self.get_mut() {
            Self::Direct(ref mut reader) => Pin::new(reader).poll_read(cx, buf),
            Self::Zlib(ref mut reader) => Pin::new(reader).poll_read(cx, buf),
        }
    }
}

/// Array attribute values iterator for `{i,f}{32,64}` array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ArrayAttributeValues<R, E> {
    /// Decoded reader.
    reader: R,
    // `total_elements`: unused.
    ///// Number of total elements.
    //total_elements: u32,
    /// Number of rest elements.
    rest_elements: u32,
    /// Whether an error is happened.
    has_error: bool,
    /// Element type.
    _element_type: PhantomData<E>,
}

impl<R, E> ArrayAttributeValues<R, E>
{
    /// Creates a new `ArrayAttributeValues`.
    pub(crate) fn new(reader: R, total_elements: u32) -> Self {
        Self {
            reader,
            //total_elements,
            rest_elements: total_elements,
            has_error: false,
            _element_type: PhantomData,
        }
    }

    /// Returns whether an error happened or not.
    pub(crate) fn has_error(&self) -> bool {
        self.has_error
    }
}

/// Implement common traits for `ArrayAttributeValues`.
macro_rules! impl_array_attr_values {
    ($ty_elem:ty, $read_elem:ident) => {
        impl<R> Stream for ArrayAttributeValues<R, $ty_elem> 
        where
            R: AsyncByteOrderRead + Unpin + Send
        {
            type Item = Result<$ty_elem>;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
                let this = self.get_mut();
                if this.rest_elements == 0 {
                    return Poll::Ready(None);
                }
                Poll::Ready(match ready!(<$ty_elem>::from_async_reader(&mut this.reader).poll(cx)) {
                    Ok(v) => {
                        this.rest_elements = this
                            .rest_elements
                            .checked_sub(1)
                            .expect("This should be executed only when there are rest elements");
                        Some(Ok(v))
                    }
                    Err(e) => {
                        this.has_error = true;
                        Some(Err(e.into()))
                    }
                })
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                (0, Some(self.rest_elements as usize))
            }
        }

        // impl<R: io::Read> std::iter::FusedIterator for ArrayAttributeValues<R, $ty_elem> {}
    };
}

impl_array_attr_values! { i32, read_i32 }
impl_array_attr_values! { i64, read_i64 }
impl_array_attr_values! { f32, read_f32 }
impl_array_attr_values! { f64, read_f64 }

/// Array attribute values iterator for `bool` array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BooleanArrayAttributeValues<R> {
    /// Decoded reader.
    reader: R,
    // `total_elements`: unused.
    ///// Number of total elements.
    //total_elements: u32,
    /// Number of rest elements.
    rest_elements: u32,
    /// Whether an error is happened.
    has_error: bool,
    /// Whether the attribute has incorrect boolean value representation.
    has_incorrect_boolean_value: bool,
}

impl<R> BooleanArrayAttributeValues<R> {
    /// Creates a new `BooleanArrayAttributeValues`.
    pub(crate) fn new(reader: R, total_elements: u32) -> Self {
        Self {
            reader,
            //total_elements,
            rest_elements: total_elements,
            has_error: false,
            has_incorrect_boolean_value: false,
        }
    }

    /// Returns whether the attribute has incorrect boolean value
    /// representation.
    pub(crate) fn has_incorrect_boolean_value(&self) -> bool {
        self.has_incorrect_boolean_value
    }

    /// Returns whether an error happened or not.
    pub(crate) fn has_error(&self) -> bool {
        self.has_error
    }
}

impl<R: AsyncByteOrderRead + Unpin + Send> Stream for BooleanArrayAttributeValues<R> {
    type Item = Result<bool>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.rest_elements == 0 {
            return Poll::Ready(None);
        }
        Poll::Ready(match ready!(this.reader.read_u8().boxed().poll(cx)) {
            Ok(raw) => {
                this.rest_elements = this
                    .rest_elements
                    .checked_sub(1)
                    .expect("This should be executed only when there are rest elements");
                if raw != b'T' && raw != b'Y' {
                    this.has_incorrect_boolean_value = true;
                }
                let v = (raw & 1) != 0;
                Some(Ok(v))
            }
            Err(e) => {
                this.has_error = true;
                Some(Err(e.into()))
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.rest_elements as usize))
    }
}

// impl<R: io::Read> std::iter::FusedIterator for BooleanArrayAttributeValues<R> {}
