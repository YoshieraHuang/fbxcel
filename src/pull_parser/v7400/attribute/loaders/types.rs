//! Attribute type loader.

use crate::pull_parser::{v7400::LoadAttribute, Result};
use async_trait::async_trait;
use fbxcel_low::v7400::AttributeType;
use futures_core::Stream;
use futures_lite::AsyncRead;

/// Loader for node attribute type ([`AttributeType`]).
///
/// This returns only node attribute type ([`AttributeType`]) and discands
/// its real value.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeLoader;

#[async_trait]
impl LoadAttribute for TypeLoader {
    type Output = AttributeType;

    fn expecting(&self) -> String {
        "any type".into()
    }

    async fn load_bool(self, _: bool) -> Result<Self::Output> {
        Ok(AttributeType::Bool)
    }

    async fn load_i16(self, _: i16) -> Result<Self::Output> {
        Ok(AttributeType::I16)
    }

    async fn load_i32(self, _: i32) -> Result<Self::Output> {
        Ok(AttributeType::I32)
    }

    async fn load_i64(self, _: i64) -> Result<Self::Output> {
        Ok(AttributeType::I64)
    }

    async fn load_f32(self, _: f32) -> Result<Self::Output> {
        Ok(AttributeType::F32)
    }

    async fn load_f64(self, _: f64) -> Result<Self::Output> {
        Ok(AttributeType::F64)
    }

    async fn load_seq_bool(
        self,
        _: impl Stream<Item = Result<bool>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeType::ArrBool)
    }

    async fn load_seq_i32(
        self,
        _: impl Stream<Item = Result<i32>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeType::ArrI32)
    }

    async fn load_seq_i64(
        self,
        _: impl Stream<Item = Result<i64>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeType::ArrI64)
    }

    async fn load_seq_f32(
        self,
        _: impl Stream<Item = Result<f32>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeType::ArrF32)
    }

    async fn load_seq_f64(
        self,
        _: impl Stream<Item = Result<f64>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Ok(AttributeType::ArrF64)
    }

    async fn load_binary(self, _: impl AsyncRead + Send + 'async_trait + Unpin, _len: u64) -> Result<Self::Output> {
        Ok(AttributeType::Binary)
    }

    async fn load_string(self, _: impl AsyncRead + Send + 'async_trait + Unpin, _len: u64) -> Result<Self::Output> {
        Ok(AttributeType::String)
    }
}
