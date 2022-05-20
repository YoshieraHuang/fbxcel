//! Node attribute loader.

use async_trait::async_trait;
use futures_core::Stream;
use futures_lite::AsyncBufRead;

use crate::{error::DataError, Result};

/// A trait for attribute loader types.
///
/// This is a lot like a "visitor", but node attributes do not have recursive
/// structures, so this loader is not "visitor".
///
/// The `load_*` method corresponding to the node attribute type are called with
/// its value.
///
/// All of `load_*` has default implementation to return error as "unexpected
/// attribute".
/// Users should implement them manually for types they want to interpret.
///
/// For simple types, [`pull_parser::v7400::attribute::loaders`][`super::loaders`] module contains
/// useful loaders.
#[async_trait]
pub trait LoadAttribute: Sized + std::fmt::Debug {
    /// Result type on successful read.
    type Output;

    /// Describes the expecting value.
    fn expecting(&self) -> String;

    /// Loads boolean value.
    async fn load_bool(self, _: bool) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "boolean".into()).into())
    }

    /// Loads `i16` value.
    async fn load_i16(self, _: i16) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "i16".into()).into())
    }

    /// Loads `i32` value.
    async fn load_i32(self, _: i32) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "i32".into()).into())
    }

    /// Loads `i64` value.
    async fn load_i64(self, _: i64) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "i64".into()).into())
    }

    /// Loads `f32` value.
    async fn load_f32(self, _: f32) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "f32".into()).into())
    }

    /// Loads `f64` value.
    async fn load_f64(self, _: f64) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "f64".into()).into())
    }

    /// Loads boolean array.
    async fn load_seq_bool(
        self,
        _: impl Stream<Item = Result<bool>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "boolean array".into()).into())
    }

    /// Loads `i32` array.
    async fn load_seq_i32(
        self,
        _: impl Stream<Item = Result<i32>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "i32 array".into()).into())
    }

    /// Loads `i64` array.
    async fn load_seq_i64(
        self,
        _: impl Stream<Item = Result<i64>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "i64 array".into()).into())
    }

    /// Loads `f32` array.
    async fn load_seq_f32(
        self,
        _: impl Stream<Item = Result<f32>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "f32 array".into()).into())
    }

    /// Loads `f64` array.
    async fn load_seq_f64(
        self,
        _: impl Stream<Item = Result<f64>> + Send + 'async_trait,
        _len: usize,
    ) -> Result<Self::Output> {
        Err(DataError::UnexpectedAttribute(self.expecting(), "f64 array".into()).into())
    }

    /// Loads binary value on buffered reader.
    ///
    /// This method should return error when the given reader returned error.
    async fn load_binary(
        self,
        reader: impl AsyncBufRead + Send + 'async_trait + Unpin,
        len: u64,
    ) -> Result<Self::Output> {
        self.load_binary(reader, len).await
    }

    /// Loads string value on buffered reader.
    ///
    /// This method should return error when the given reader returned error.
    async fn load_string(
        self,
        reader: impl AsyncBufRead + Send + 'async_trait + Unpin,
        len: u64,
    ) -> Result<Self::Output> {
        self.load_string(reader, len).await
    }
}
