//! Types and functions for all supported versions.

use async_position_reader::{AsyncPositionRead, SeekableReader, SimpleReader};
use fbxcel_low::{FbxHeader, FbxVersion};
use futures_lite::{AsyncRead, AsyncSeek};

use crate::pull_parser::{self, ParserVersion};

pub use self::error::{Error, Result};

mod error;

/// FBX tree type with any supported version.
#[non_exhaustive]
pub enum AnyParser<R> {
    /// FBX 7.4 or later.
    V7400(super::v7400::Parser<R>),
}

impl<R: AsyncPositionRead> AnyParser<R> {
    /// Returns the parser version.
    pub fn parser_version(&self) -> ParserVersion {
        match self {
            AnyParser::V7400(_) => pull_parser::v7400::Parser::<R>::PARSER_VERSION,
        }
    }

    /// Returns the FBX version.
    pub fn fbx_version(&self) -> FbxVersion {
        match self {
            AnyParser::V7400(parser) => parser.fbx_version(),
        }
    }
}

/// Returns the parser version for the FBX data.
fn parser_version(header: FbxHeader) -> Result<ParserVersion> {
    ParserVersion::from_fbx_version(header.version())
        .ok_or_else(|| Error::UnsupportedVersion(header.version()))
}

/// Loads a tree from the given reader.
///
/// This works for seekable readers (which implement [`std::io::Seek`]), but
/// [`from_seekable_reader`] should be used for them, because it is more
/// efficent.
pub async fn from_reader<R: AsyncRead + Unpin + Send>(
    mut reader: R,
) -> Result<AnyParser<SimpleReader<R>>> {
    let header = FbxHeader::load(&mut reader).await?;
    match parser_version(header)? {
        ParserVersion::V7400 => {
            let parser = pull_parser::v7400::from_reader(header, reader).unwrap_or_else(|e| {
                panic!(
                    "Should never fail: FBX version {:?} should be supported by v7400 parser: {}",
                    header.version(),
                    e
                )
            });
            Ok(AnyParser::V7400(parser))
        }
    }
}

/// Loads a tree from the given seekable reader.
pub async fn from_seekable_reader<R: AsyncRead + AsyncSeek + Unpin + Send>(
    mut reader: R,
) -> Result<AnyParser<SeekableReader<R>>> {
    let header = FbxHeader::load(&mut reader).await?;
    match parser_version(header)? {
        ParserVersion::V7400 => {
            let parser =
                pull_parser::v7400::from_seekable_reader(header, reader).unwrap_or_else(|e| {
                    panic!(
                    "Should never fail: FBX version {:?} should be supported by v7400 parser: {}",
                    header.version(),
                    e
                )
                });
            Ok(AnyParser::V7400(parser))
        }
    }
}
