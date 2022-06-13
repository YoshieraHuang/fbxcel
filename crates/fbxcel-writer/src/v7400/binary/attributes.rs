//! Node attributes writer.

use std::{
    convert::{Infallible, TryFrom},
    io::SeekFrom,
};

use crate::v7400::binary::{Error, Result, Writer};
use fbxcel_low::v7400::{ArrayAttributeEncoding, ArrayAttributeHeader, AttributeType};
use futures_util::{io, AsyncRead, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};

use super::stream_position::StreamPosition;

mod array;

/// A trait for types which can be represented as single bytes array.
pub(crate) trait IntoBytes: Sized {
    type Bytes;
    /// Calls the given function with the bytes array.
    fn into_bytes(self) -> Self::Bytes;
}

impl IntoBytes for bool {
    type Bytes = [u8; 1];

    fn into_bytes(self) -> Self::Bytes {
        let v = if self { b'Y' } else { b'T' };
        v.to_le_bytes()
    }
}

impl IntoBytes for i16 {
    type Bytes = [u8; 2];

    fn into_bytes(self) -> Self::Bytes {
        self.to_le_bytes()
    }
}

impl IntoBytes for i32 {
    type Bytes = [u8; 4];
    fn into_bytes(self) -> Self::Bytes {
        self.to_le_bytes()
    }
}

impl IntoBytes for i64 {
    type Bytes = [u8; 8];

    fn into_bytes(self) -> Self::Bytes {
        self.to_le_bytes()
    }
}

impl IntoBytes for f32 {
    type Bytes = [u8; 4];

    fn into_bytes(self) -> Self::Bytes {
        self.to_bits().to_le_bytes()
    }
}

impl IntoBytes for f64 {
    type Bytes = [u8; 8];

    fn into_bytes(self) -> Self::Bytes {
        self.to_bits().to_le_bytes()
    }
}

/// Node attributes writer.
///
/// See [module documentation](index.html) for usage.
pub struct AttributesWriter<'a, W> {
    /// Inner writer.
    writer: &'a mut Writer<W>,
}

/// Implement `append_*` methods for single value.
macro_rules! impl_single_attr_append {
    ($(
        $(#[$meta:meta])*
        $method:ident($ty:ty): $variant:ident;
    )*) => {
        $(
            $(#[$meta])*
            pub async fn $method(&mut self, v: $ty) -> Result<()>
            where
                W: AsyncWrite + Unpin
            {
                self.update_node_header()?;
                self.write_type_code(AttributeType::$variant).await?;
                let ref bytes = v.into_bytes();
                self.writer.sink().write_all(bytes).await.map_err(Into::into)
            }
        )*
    }
}

/// Implement `append_*` methods for array values.
macro_rules! impl_arr_from_iter {
    ($(
        $(#[$meta:meta])*
        $name:ident: $ty_elem:ty {
            from_result_iter: $name_from_result_iter:ident,
            tyval: $tyval:ident,
        },
    )*) => {$(
        $(#[$meta])*
        pub async fn $name(
            &mut self,
            encoding: impl Into<Option<ArrayAttributeEncoding>>,
            iter: impl IntoIterator<Item = $ty_elem>,
        ) -> Result<()>
        where
            W: AsyncWrite + AsyncSeek + Unpin
        {
            array::write_array_attr_result_iter(
                self,
                AttributeType::$tyval,
                encoding.into(),
                iter.into_iter().map(Ok::<_, Infallible>),
            ).await
        }

        $(#[$meta])*
        pub async fn $name_from_result_iter<E>(
            &mut self,
            encoding: impl Into<Option<ArrayAttributeEncoding>>,
            iter: impl IntoIterator<Item = std::result::Result<$ty_elem, E>>,
        ) -> Result<()>
        where
            W: AsyncWrite + AsyncSeek + Unpin,
            E: Into<Box<dyn std::error::Error + 'static>>,
        {
            array::write_array_attr_result_iter(
                self,
                AttributeType::$tyval,
                encoding.into(),
                iter.into_iter().map(|res| res.map_err(|e| Error::UserDefined(e.into()))),
            ).await
        }
    )*}
}

impl<'a, W> AttributesWriter<'a, W> {
    /// Creates a new `AttributesWriter`.
    pub(crate) fn new(writer: &'a mut Writer<W>) -> Self {
        Self { writer }
    }

    /// Returns the inner writer.
    pub(crate) fn sink(&mut self) -> &mut W {
        self.writer.sink()
    }

    /// Writes the given attribute type as type code.
    async fn write_type_code(&mut self, ty: AttributeType) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.writer
            .sink()
            .write_all(&ty.type_code().to_le_bytes())
            .await
            .map_err(Into::into)
    }

    /// Updates the node header.
    fn update_node_header(&mut self) -> Result<()> {
        let node_header = self
            .writer
            .current_node_header()
            .expect("Should never fail: some nodes must be open if `AttributesWriter` exists");
        node_header.num_attributes =
            node_header
                .num_attributes
                .checked_add(1)
                .ok_or(Error::TooManyAttributes(
                    node_header.num_attributes as usize,
                ))?;

        Ok(())
    }

    impl_single_attr_append! {
        /// Writes a single boolean attribute.
        append_bool(bool): Bool;
        /// Writes a single `i16` attribute.
        append_i16(i16): I16;
        /// Writes a single `i32` attribute.
        append_i32(i32): I32;
        /// Writes a single `i64` attribute.
        append_i64(i64): I64;
        /// Writes a single `f32` attribute.
        append_f32(f32): F32;
        /// Writes a single `f64` attribute.
        append_f64(f64): F64;
    }

    /// Writes the given array attribute header.
    async fn write_array_header(&mut self, header: &ArrayAttributeHeader) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        array::write_array_header(self.writer.sink(), header).await?;
        Ok(())
    }

    /// Writes some headers for an array attibute, and returns header position.
    pub(crate) async fn initialize_array(
        &mut self,
        ty: AttributeType,
        encoding: ArrayAttributeEncoding,
    ) -> Result<u64>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        self.update_node_header()?;

        // Write attribute header.
        self.write_type_code(ty).await?;
        let header_pos = self.writer.sink().stream_position().await?;

        // Write array header placeholder.
        self.write_array_header(&ArrayAttributeHeader {
            elements_count: 0,
            encoding,
            bytelen: 0,
        })
        .await?;

        Ok(header_pos)
    }

    /// Updates an array attribute header.
    ///
    /// Note that this should be called at the end of the array attribute.
    async fn finalize_array(&mut self, header_pos: u64, header: &ArrayAttributeHeader) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        // Write real array header.
        let end_pos = self.writer.sink().stream_position().await?;
        self.writer.sink().seek(SeekFrom::Start(header_pos)).await?;
        self.write_array_header(header).await?;
        self.writer.sink().seek(SeekFrom::Start(end_pos)).await?;

        Ok(())
    }

    impl_arr_from_iter! {
        /// Writes a boolean array attribute.
        append_arr_bool_from_iter: bool {
            from_result_iter: append_arr_bool_from_result_iter,
            tyval: ArrBool,
        },

        /// Writes an `i32` array attribute.
        append_arr_i32_from_iter: i32 {
            from_result_iter: append_arr_i32_from_result_iter,
            tyval: ArrI32,
        },

        /// Writes an `i64` array attribute.
        append_arr_i64_from_iter: i64 {
            from_result_iter: append_arr_i64_from_result_iter,
            tyval: ArrI64,
        },

        /// Writes an `f32` array attribute.
        append_arr_f32_from_iter: f32 {
            from_result_iter: append_arr_f32_from_result_iter,
            tyval: ArrI32,
        },

        /// Writes an `f64` array attribute.
        append_arr_f64_from_iter: f64 {
            from_result_iter: append_arr_f64_from_result_iter,
            tyval: ArrI64,
        },
    }

    /// Writes some headers for a special attribute, and returns the special
    /// header position.
    async fn initialize_special(&mut self, ty: AttributeType) -> Result<u64>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        self.update_node_header()?;

        // Write attribute header.
        self.write_type_code(ty).await?;

        // Write special attribute header (dummy).
        let header_pos = self.writer.sink().stream_position().await?;
        self.writer.sink().write_all(&0u32.to_le_bytes()).await?;

        Ok(header_pos)
    }

    /// Updates an array attribute header.
    ///
    /// Note that this should be called at the end of the array attribute.
    async fn finalize_special(&mut self, header_pos: u64, bytelen: usize) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        // Calculate header fields.
        let bytelen = u32::try_from(bytelen).map_err(|_| Error::AttributeTooLong(bytelen))?;

        // Write real special attribute header.
        let end_pos = self.writer.sink().stream_position().await?;
        self.writer.sink().seek(SeekFrom::Start(header_pos)).await?;
        self.writer.sink().write_all(&bytelen.to_le_bytes()).await?;
        self.writer.sink().seek(SeekFrom::Start(end_pos)).await?;

        Ok(())
    }

    /// Writes a binary attribute.
    pub async fn append_binary_direct(&mut self, binary: &[u8]) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        let header_pos = self.initialize_special(AttributeType::Binary).await?;

        self.writer.sink().write_all(binary).await?;

        self.finalize_special(header_pos, binary.len()).await?;

        Ok(())
    }

    /// Writes a string attribute.
    pub async fn append_string_direct(&mut self, string: &str) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        let header_pos = self.initialize_special(AttributeType::String).await?;

        self.writer.sink().write_all(string.as_ref()).await?;

        self.finalize_special(header_pos, string.len()).await?;

        Ok(())
    }

    /// Writes a binary attribute read from the given reader.
    pub async fn append_binary_from_reader(
        &mut self,
        mut reader: impl AsyncRead + Unpin,
    ) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        let header_pos = self.initialize_special(AttributeType::Binary).await?;

        // Write bytes.
        let written_len = io::copy(&mut reader, self.writer.sink()).await?;

        self.finalize_special(header_pos, written_len as usize)
            .await?;

        Ok(())
    }

    /// Writes a binary attribute from the given iterator.
    pub async fn append_binary_from_iter(
        &mut self,
        iter: impl IntoIterator<Item = u8>,
    ) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        let header_pos = self.initialize_special(AttributeType::Binary).await?;

        let mut len = 0usize;
        for v in iter.into_iter() {
            self.writer.sink().write_all(&[v]).await?;
            len = len
                .checked_add(1)
                .ok_or(Error::AttributeTooLong(std::usize::MAX))?;
        }

        self.finalize_special(header_pos, len).await?;

        Ok(())
    }

    /// Writes a binary attribute from the given iterator.
    pub async fn append_binary_from_result_iter<E>(
        &mut self,
        iter: impl IntoIterator<Item = std::result::Result<u8, E>>,
    ) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
        E: Into<Box<dyn std::error::Error + 'static>>,
    {
        let header_pos = self.initialize_special(AttributeType::Binary).await?;

        let mut len = 0usize;
        for v in iter.into_iter() {
            let v = v.map_err(|e| Error::UserDefined(e.into()))?;
            self.writer.sink().write_all(&[v]).await?;
            len = len
                .checked_add(1)
                .ok_or(Error::AttributeTooLong(std::usize::MAX))?;
        }

        self.finalize_special(header_pos, len).await?;

        Ok(())
    }

    /// Writes a string attribute from the given iterator.
    pub async fn append_string_from_iter(
        &mut self,
        iter: impl IntoIterator<Item = char>,
    ) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        let header_pos = self.initialize_special(AttributeType::String).await?;

        let buf = &mut [0u8; 4];
        let mut len = 0usize;
        for c in iter.into_iter() {
            let char_len = c.encode_utf8(buf).len();
            self.writer.sink().write_all(buf).await?;
            len = len
                .checked_add(char_len)
                .ok_or(Error::AttributeTooLong(std::usize::MAX))?;
        }

        self.finalize_special(header_pos, len).await?;

        Ok(())
    }

    /// Writes a string attribute from the given iterator.
    pub async fn append_string_from_result_iter<E>(
        &mut self,
        iter: impl IntoIterator<Item = std::result::Result<char, E>>,
    ) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
        E: Into<Box<dyn std::error::Error + 'static>>,
    {
        let header_pos = self.initialize_special(AttributeType::String).await?;

        let buf = &mut [0u8; 4];
        let mut len = 0usize;
        for c in iter.into_iter() {
            let c = c.map_err(|e| Error::UserDefined(e.into()))?;
            let char_len = c.encode_utf8(buf).len();
            self.writer.sink().write_all(buf).await?;
            len = len
                .checked_add(char_len)
                .ok_or(Error::AttributeTooLong(std::usize::MAX))?;
        }

        self.finalize_special(header_pos, len).await?;

        Ok(())
    }
}
