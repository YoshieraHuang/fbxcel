//! Binary writer error.

use std::{convert::Infallible, io};

use thiserror::Error;

use fbxcel_low::FbxVersion;

/// Write result.
pub type Result<T> = std::result::Result<T, Error>;

/// Write error.
#[derive(Debug, Error)]
pub enum Error {
    /// Node attribute is too long.
    #[error("Node attribute is too long: {0} bytes")]
    AttributeTooLong(usize),
    /// Compression error.
    #[error(transparent)]
    Compression(#[from] CompressionError),
    /// File is too large.
    #[error("File is too large: {0} bytes")]
    FileTooLarge(u64),
    /// I/O error.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// There are no nodes to close.
    #[error("No nodes to close")]
    NoNodesToClose,
    /// Node name is too long.
    #[error("Node name is too long: {0} bytes")]
    NodeNameTooLong(usize),
    /// Too many array attribute elements.
    #[error("Too many array elements for a single node attribute: count={0}")]
    TooManyArrayAttributeElements(usize),
    /// Too many attributes.
    #[error("Too many attributes: count={0}")]
    TooManyAttributes(usize),
    /// There remains unclosed nodes.
    #[error("There remains unclosed nodes: depth={0}")]
    UnclosedNode(usize),
    /// Unsupported FBX version.
    #[error("Unsupported FBX version: {0:?}")]
    UnsupportedFbxVersion(FbxVersion),
    /// User-defined error.
    #[error("User defined: {0}")]
    UserDefined(Box<dyn std::error::Error + 'static>),
}

// impl error::Error for Error {
//     fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//         match self {
//             Error::Compression(e) => Some(e),
//             Error::Io(e) => Some(e),
//             Error::UserDefined(e) => Some(&**e),
//             _ => None,
//         }
//     }
// }

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Error::AttributeTooLong(v) => write!(f, "Node attribute is too long: {} bytes", v),
//             Error::Compression(e) => write!(f, "Compression error: {}", e),
//             Error::FileTooLarge(v) => write!(f, "File is too large: {} bytes", v),
//             Error::Io(e) => write!(f, "I/O error: {}", e),
//             Error::NoNodesToClose => write!(f, "There are no nodes to close"),
//             Error::NodeNameTooLong(v) => write!(f, "Node name is too long: {} bytes", v),
//             Error::TooManyArrayAttributeElements(v) => write!(
//                 f,
//                 "Too many array elements for a single node attribute: count={}",
//                 v
//             ),
//             Error::TooManyAttributes(v) => write!(f, "Too many attributes: count={}", v),
//             Error::UnclosedNode(v) => write!(f, "There remains unclosed nodes: depth={}", v),
//             Error::UnsupportedFbxVersion(v) => write!(f, "Unsupported FBX version: {:?}", v),
//             Error::UserDefined(e) => write!(f, "User-defined error: {}", e),
//         }
//     }
// }

// impl From<io::Error> for Error {
//     fn from(e: io::Error) -> Self {
//         Error::Io(e)
//     }
// }

// impl From<CompressionError> for Error {
//     fn from(e: CompressionError) -> Self {
//         Error::Compression(e)
//     }
// }

/// Compression error.
#[derive(Debug, Error)]
pub enum CompressionError {
    /// Zlib error
    #[error("Zlib error: {0}")]
    Zlib(
        #[from]
        #[source]
        io::Error,
    ),
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        panic!("infallible")
    }
}
