//! Single type loader.

use crate::{v7400::LoadAttribute, Result};
use async_trait::async_trait;
use futures_util::{AsyncBufRead, AsyncReadExt, Stream, TryStreamExt};

/// Loader for primitive types.
///
/// Supported types are: `bool`, `i16` , `i32`, `i64`, `f32`, and `f64`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimitiveLoader<T>(std::marker::PhantomData<T>);

/// Generates `LoadAttribute` implementations for `PrimitiveLoader<T>`.
macro_rules! impl_load_attribute_for_primitives {
    ($ty:ty, $method_name:ident, $expecting_type:expr) => {
        #[async_trait]
        impl LoadAttribute for PrimitiveLoader<$ty> {
            type Output = $ty;

            fn expecting(&self) -> String {
                $expecting_type.into()
            }

            async fn $method_name(self, v: $ty) -> Result<Self::Output> {
                Ok(v)
            }
        }
    };
}

impl_load_attribute_for_primitives!(bool, load_bool, "single boolean");
impl_load_attribute_for_primitives!(i16, load_i16, "single i16");
impl_load_attribute_for_primitives!(i32, load_i32, "single i32");
impl_load_attribute_for_primitives!(i64, load_i64, "single i64");
impl_load_attribute_for_primitives!(f32, load_f32, "single f32");
impl_load_attribute_for_primitives!(f64, load_f64, "single f64");

/// Loader for array types.
///
/// Supported types are: `Vec<{bool, i32, i64, f32, f64}>`.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayLoader<T>(std::marker::PhantomData<T>);

/// Generates `LoadAttribute` implementations for `PrimitiveLoader<T>`.
macro_rules! impl_load_attribute_for_arrays {
    ($ty:ty, $method_name:ident, $expecting_type:expr) => {
        #[async_trait]
        impl LoadAttribute for ArrayLoader<Vec<$ty>> {
            type Output = Vec<$ty>;

            fn expecting(&self) -> String {
                $expecting_type.into()
            }

            async fn $method_name(
                self,
                iter: impl Stream<Item = Result<$ty>> + Send + 'async_trait,
                _: usize,
            ) -> Result<Self::Output> {
                Ok(iter.try_collect().await?)
            }
        }
    };
}

impl_load_attribute_for_arrays!(bool, load_seq_bool, "boolean array");
impl_load_attribute_for_arrays!(i32, load_seq_i32, "i32 array");
impl_load_attribute_for_arrays!(i64, load_seq_i64, "i64 array");
impl_load_attribute_for_arrays!(f32, load_seq_f32, "f32 array");
impl_load_attribute_for_arrays!(f64, load_seq_f64, "f64 array");

/// Loader for a binary.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BinaryLoader;

#[async_trait]
impl LoadAttribute for BinaryLoader {
    type Output = Vec<u8>;

    fn expecting(&self) -> String {
        "binary".into()
    }

    async fn load_binary(
        self,
        mut reader: impl AsyncBufRead + Send + 'async_trait + Unpin,
        len: u64,
    ) -> Result<Self::Output> {
        let mut buf = Vec::with_capacity(len as usize);
        reader.read_to_end(&mut buf).await?;
        Ok(buf)
    }
}

/// Loader for a string.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringLoader;

#[async_trait]
impl LoadAttribute for StringLoader {
    type Output = String;

    fn expecting(&self) -> String {
        "string".into()
    }

    async fn load_string(
        self,
        mut reader: impl AsyncBufRead + Send + 'async_trait + Unpin,
        len: u64,
    ) -> Result<Self::Output> {
        let mut buf = String::with_capacity(len as usize);
        reader.read_to_string(&mut buf).await?;
        Ok(buf)
    }
}
