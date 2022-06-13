//! Types and functions for all supported versions.
//!
//! To see how to use `AnyDocument`, see [crate-level
//! documentation](../index.html).

use fbxcel_low::FbxVersion;
use fbxcel_tree::any::AnyTree;
use futures_util::{AsyncBufRead, AsyncRead, AsyncSeek};

pub use self::error::{Error, Result};

mod error;

/// FBX tree type with any supported version.
#[non_exhaustive]
pub enum AnyDocument {
    /// FBX 7.4 or later.
    V7400(FbxVersion, Box<crate::v7400::Document>),
}

impl AnyDocument {
    /// Loads a document from the given seekable reader.
    pub async fn from_seekable_reader(
        reader: impl AsyncRead + AsyncSeek + AsyncBufRead + Unpin + Send,
    ) -> Result<Self> {
        match AnyTree::from_seekable_reader(reader).await? {
            AnyTree::V7400(fbx_version, tree, _footer) => {
                let doc = crate::v7400::Loader::new().load_from_tree(tree)?;
                Ok(AnyDocument::V7400(fbx_version, Box::new(doc)))
            }
            tree => Err(Error::UnsupportedVersion(tree.fbx_version())),
        }
    }

    /// Returns the FBX version of the loaded document.
    pub fn fbx_version(&self) -> FbxVersion {
        match self {
            Self::V7400(ver, _) => *ver,
        }
    }
}
