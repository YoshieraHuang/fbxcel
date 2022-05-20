//! Direct attribute value loader.

use crate::{v7400::LoadAttribute, Result};
use async_trait::async_trait;
use fbxcel_low::v7400::AttributeValue;
use futures_core::Stream;
use futures_lite::{AsyncBufRead, AsyncReadExt, StreamExt};

/// Loader for [`AttributeValue`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirectLoader;

#[async_trait]
impl LoadAttribute for DirectLoader {
    type Output = AttributeValue;

    fn expecting(&self) -> String {
        "any type".into()
    }

    async fn load_bool(self, v: bool) -> Result<Self::Output> {
        Ok(AttributeValue::Bool(v))
    }

    async fn load_i16(self, v: i16) -> Result<Self::Output> {
        Ok(AttributeValue::I16(v))
    }

    async fn load_i32(self, v: i32) -> Result<Self::Output> {
        Ok(AttributeValue::I32(v))
    }

    async fn load_i64(self, v: i64) -> Result<Self::Output> {
        Ok(AttributeValue::I64(v))
    }

    async fn load_f32(self, v: f32) -> Result<Self::Output> {
        Ok(AttributeValue::F32(v))
    }

    async fn load_f64(self, v: f64) -> Result<Self::Output> {
        Ok(AttributeValue::F64(v))
    }

    async fn load_seq_bool(
        self,
        iter: impl Stream<Item = Result<bool>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeValue::ArrBool(iter.try_collect().await?))
    }

    async fn load_seq_i32(
        self,
        iter: impl Stream<Item = Result<i32>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeValue::ArrI32(iter.try_collect().await?))
    }

    async fn load_seq_i64(
        self,
        iter: impl Stream<Item = Result<i64>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeValue::ArrI64(iter.try_collect().await?))
    }

    async fn load_seq_f32(
        self,
        iter: impl Stream<Item = Result<f32>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeValue::ArrF32(iter.try_collect().await?))
    }

    async fn load_seq_f64(
        self,
        iter: impl Stream<Item = Result<f64>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeValue::ArrF64(iter.try_collect().await?))
    }

    async fn load_binary(
        self,
        mut reader: impl AsyncBufRead + Send + 'async_trait + Unpin,
        len: u64,
    ) -> Result<Self::Output> {
        let mut buf = Vec::with_capacity(len as usize);
        reader.read_to_end(&mut buf).await?;
        Ok(AttributeValue::Binary(buf))
    }

    async fn load_string(
        self,
        mut reader: impl AsyncBufRead + Send + 'async_trait + Unpin,
        len: u64,
    ) -> Result<Self::Output> {
        let mut buf = String::with_capacity(len as usize);
        reader.read_to_string(&mut buf).await?;
        Ok(AttributeValue::String(buf))
    }
}
