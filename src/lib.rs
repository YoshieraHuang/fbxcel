//! The excellent FBX library.
//!
//! [`low`] module provides low-level data types such as FBX header, node
//! attribute value, etc.
//!
//! [`pull_parser`] module provides pull parser for FBX binary format.
//! ASCII format is not supported.
//!
//! [`tree`] module provides tree types, which allow users to access FBX data as
//! tree, not as stream of parser events.
//! To use `tree` module, enable `tree` feature.
//!
//! [`writer`] module provides writer types.
//! To use `writer` module, enable `writer` feature.
#![cfg_attr(feature = "docsrs", feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use fbxcel_low as low;
pub use fbxcel_pull_parser as pull_parser;
// pub mod pull_parser;
#[cfg(feature = "tree")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "tree")))]
pub use fbxcel_tree as tree;

#[cfg(feature = "writer")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
pub use fbxcel_writer as writer;

#[cfg(feature = "dom")]
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "dom")))]
pub use fbxcel_dom as dom;
