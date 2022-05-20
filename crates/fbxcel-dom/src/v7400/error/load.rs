//! FBX DOM load error.

use std::{error, fmt};

use fbxcel_tree::v7400::LoadError as TreeLoadError;
use thiserror::Error;

/// FBX DOM load error.
#[derive(Debug)]
pub struct LoadError(Box<dyn error::Error + Send + Sync + 'static>);

impl LoadError {
    /// Creates a new `LoadError`.
    pub(crate) fn new(e: impl Into<Box<dyn error::Error + Send + Sync + 'static>>) -> Self {
        Self(e.into())
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for LoadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&*self.0)
    }
}

impl From<TreeLoadError> for LoadError {
    fn from(e: TreeLoadError) -> Self {
        Self::new(e)
    }
}

impl From<StructureError> for LoadError {
    fn from(e: StructureError) -> Self {
        Self::new(e)
    }
}

/// FBX DOM structure error.
#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum StructureError {
    /// Toplevel `Connections` node not found.
    #[error("Toplevel `Connections` node not found")]
    MissingConnectionsNode,
    /// Toplevel `Documents` node not found.
    #[error("Toplevel `Documents` node not found")]
    MissingDocumentsNode,
    /// Toplevel `Objects` node not found.
    #[error("Toplevel `Objects` node not found")]
    MissingObjectsNode,
}
