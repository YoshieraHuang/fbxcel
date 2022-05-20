//! Reader functions and traits.

// use async_trait::async_trait;
// use futures_lite::io::AsyncRead;

// use byte_order_reader::AsyncByteOrderRead;

use super::Parser;
use crate::{error::DataError, Result, SyntacticPosition, Warning};
use async_position_reader::AsyncPositionRead;
use async_trait::async_trait;
use byte_order_reader::FromAsyncReader;
use byteorder::{ByteOrder, LE};
use fbxcel_low::{
    v7400::{AttributeType, FbxFooter, NodeHeader, SpecialAttributeHeader},
    FbxVersion,
};
use futures_lite::AsyncReadExt;
use log::debug;

/// A trait for types readable from a parser.
#[async_trait]
pub(crate) trait FromAsyncParser<R>: Sized {
    /// Reads the data from the given parser.
    async fn from_async_parser(parser: &mut Parser<R>) -> Result<Self>
    where
        R: AsyncPositionRead + Unpin + Send;
}

#[async_trait]
impl<R> FromAsyncParser<R> for NodeHeader
where
    R: AsyncPositionRead + Unpin + Send,
{
    async fn from_async_parser(parser: &mut Parser<R>) -> Result<Self> {
        let (end_offset, num_attributes, bytelen_attributes) = if parser.fbx_version().raw() < 7500
        {
            let eo = u64::from(parser.parse::<u32>().await?);
            let na = u64::from(parser.parse::<u32>().await?);
            let ba = u64::from(parser.parse::<u32>().await?);
            (eo, na, ba)
        } else {
            let eo = parser.parse::<u64>().await?;
            let na = parser.parse::<u64>().await?;
            let ba = parser.parse::<u64>().await?;
            (eo, na, ba)
        };
        let bytelen_name = parser.parse::<u8>().await?;

        Ok(Self {
            end_offset,
            num_attributes,
            bytelen_attributes,
            bytelen_name,
        })
    }
}

#[async_trait]
impl<R> FromAsyncParser<R> for FbxFooter
where
    R: AsyncPositionRead + Unpin + Send,
{
    async fn from_async_parser(parser: &mut Parser<R>) -> Result<Self> {
        let start_pos = parser.reader().position();

        // Read unknown field 1.
        let unknown1 = {
            /// Expected upper 4-bits of the unknown field 1.
            const EXPECTED: [u8; 16] = [
                0xf0, 0xb0, 0xa0, 0x00, 0xd0, 0xc0, 0xd0, 0x60, 0xb0, 0x70, 0xf0, 0x80, 0x10, 0xf0,
                0x20, 0x70,
            ];
            let mut buf = [0u8; 16];
            parser.reader().read_exact(&mut buf).await?;

            for (byte, expected) in buf.iter().zip(&EXPECTED) {
                if (byte & 0xf0) != *expected {
                    let pos = SyntacticPosition {
                        byte_pos: parser.reader().position() - 16,
                        component_byte_pos: start_pos,
                        node_path: Vec::new(),
                        attribute_index: None,
                    };
                    parser.warn(Warning::UnexpectedFooterFieldValue, pos)?;
                    break;
                }
            }

            buf
        };

        // Read padding, following 144-bytes zeroes, unknown field 2, FBX
        // version, and unknown field 3.
        let (padding_len, unknown2, version, unknown3) = {
            let buf_start_pos = parser.reader().position();

            // Expected padding length.
            let expected_padding_len = (buf_start_pos.wrapping_neg() & 0x0f) as usize;
            debug!(
                "Current position = {}, expected padding length = {}",
                buf_start_pos, expected_padding_len
            );

            /// Buffer length to load footer partially.
            // Padding (min 0) + unknown2 (4) + version (4) + zeroes (120)
            // + unknown3 (16) = 144.
            const BUF_LEN: usize = 144;
            let mut buf = [0u8; BUF_LEN];
            parser.reader().read_exact(&mut buf).await?;

            // First, get the beginning position of unknown field 3,
            // because it is expected to be starting with a non-zero byte.
            let unknown3_pos = {
                /// Start offset of search of unknown field 3.
                const SEARCH_OFFSET: usize = BUF_LEN - 16;
                let pos = (&buf[SEARCH_OFFSET..])
                    .iter()
                    .position(|&v| v != 0)
                    .ok_or(DataError::BrokenFbxFooter)?;
                SEARCH_OFFSET + pos
            };

            let padding_len = unknown3_pos & 0x0f;
            assert!(padding_len < 16);
            assert_eq!(unknown3_pos, padding_len + 128);
            let padding = &buf[..padding_len];
            let mut unknown2 = [0u8; 4];
            unknown2.copy_from_slice(&buf[padding_len..(padding_len + 4)]);
            let version_buf = &buf[(padding_len + 4)..(padding_len + 8)];
            let zeroes_120 = &buf[(padding_len + 8)..(padding_len + 128)];
            let unknown3_part = &buf[(padding_len + 128)..];

            // Check that the padding has only zeroes.
            if !padding.iter().all(|&v| v == 0) {
                return Err(DataError::BrokenFbxFooter.into());
            }

            // Check that the unknown field 2 has only zeroes.
            if unknown2 != [0u8; 4] {
                return Err(DataError::BrokenFbxFooter.into());
            }

            // Check that the FBX version is same as the FBX header.
            let version = FbxVersion::new(LE::read_u32(version_buf));
            if version != parser.fbx_version() {
                // Version mismatch.
                return Err(DataError::BrokenFbxFooter.into());
            }

            // Check that there are 120-bytes zeroes.
            if !zeroes_120.iter().all(|&v| v == 0) {
                return Err(DataError::BrokenFbxFooter.into());
            }

            // Check that the unknown field 3 has expected pattern.
            /// Expected value of unknown field 3.
            const UNKNOWN3_EXPECTED: [u8; 16] = [
                0xf8, 0x5a, 0x8c, 0x6a, 0xde, 0xf5, 0xd9, 0x7e, 0xec, 0xe9, 0x0c, 0xe3, 0x75, 0x8f,
                0x29, 0x0b,
            ];
            let mut unknown3 = [0u8; 16];
            unknown3[0..unknown3_part.len()].copy_from_slice(unknown3_part);
            parser
                .reader()
                .read_exact(&mut unknown3[unknown3_part.len()..])
                .await?;
            if unknown3 != UNKNOWN3_EXPECTED {
                return Err(DataError::BrokenFbxFooter.into());
            }

            // If the execution comes here, footer may have no error.
            // Emit warning if necessary.

            // Check if the padding has correct length.
            if padding_len != expected_padding_len {
                let pos = SyntacticPosition {
                    byte_pos: buf_start_pos,
                    component_byte_pos: start_pos,
                    node_path: Vec::new(),
                    attribute_index: None,
                };
                parser.warn(
                    Warning::InvalidFooterPaddingLength(expected_padding_len, padding_len),
                    pos,
                )?;
            }

            (padding_len, unknown2, version, unknown3)
        };

        Ok(Self {
            unknown1,
            padding_len: padding_len as u8,
            unknown2,
            fbx_version: version,
            unknown3,
        })
    }
}

macro_rules! impl_from_async_parser {
    (
        $(
            $ty:ty
        ),*
    ) => {
        $(
            #[async_trait]
            impl<R> FromAsyncParser<R> for $ty
            where
                R: AsyncPositionRead + Unpin + Send,
            {
                async fn from_async_parser(parser: &mut Parser<R>) -> Result<Self> {
                    <$ty>::from_async_reader(parser.reader())
                        .await
                        .map_err(|e| e.into())
                }
            }
        )*

    };
}

impl_from_async_parser!(
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    i128,
    f32,
    f64,
    AttributeType,
    SpecialAttributeHeader
);
