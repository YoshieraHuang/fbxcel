//! Types and functions for all supported versions.

use futures_util::{AsyncBufRead, AsyncSeek};
use log::warn;

pub use self::error::Error;
use crate::v7400;
use fbxcel_low::{self, v7400::FbxFooter, FbxVersion};
use fbxcel_pull_parser::any::AnyParser;
mod error;
use error::Result;

/// FBX tree type with any supported version.
#[non_exhaustive]
pub enum AnyTree {
    /// FBX 7.4 or later.
    V7400(FbxVersion, v7400::Tree, Result<Box<FbxFooter>>),
}

impl AnyTree {
    /// Loads a tree from the given seekable reader.
    pub async fn from_seekable_reader(
        reader: impl AsyncBufRead + AsyncSeek + Unpin + Send,
    ) -> Result<Self> {
        match fbxcel_pull_parser::any::from_seekable_reader(reader).await? {
            AnyParser::V7400(mut parser) => {
                let fbx_version = parser.fbx_version();
                parser.set_warning_handler(|w, pos| {
                    warn!("WARNING: {} (pos={:?})", w, pos);
                    Ok(())
                });
                let tree_loader = v7400::Loader::new();
                let (tree, footer) = tree_loader.load(&mut parser).await?;
                let footer = footer.map_err(|e| e.into());
                Ok(AnyTree::V7400(fbx_version, tree, footer))
            }
            _ => todo!(),
        }
    }

    /// Returns the FBX version of the document the tree came from.
    pub fn fbx_version(&self) -> FbxVersion {
        match self {
            Self::V7400(ver, _, _) => *ver,
        }
    }
}
