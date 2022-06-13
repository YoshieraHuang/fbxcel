#![feature(generic_associated_types)]
//! Low-level or primitive data types for FBX binary.

#[cfg(feature = "writer")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
pub use self::fbx_header::MAGIC;
pub use self::{
    error::LowError,
    fbx_header::{FbxHeader, HeaderError},
    version::FbxVersion,
};

mod error;
mod fbx_header;
pub mod v7400;
mod version;
