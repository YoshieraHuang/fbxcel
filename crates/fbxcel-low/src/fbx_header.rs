//! FBX binary header.

use futures_lite::{io, prelude::*};

use byte_order_reader::AsyncByteOrderRead;
use byteorder::LE;
use log::info;
use thiserror::Error;

use crate::FbxVersion;

/// Magic binary length.
const MAGIC_LEN: usize = 23;

/// Magic binary.
pub(crate) const MAGIC: &[u8; MAGIC_LEN] = b"Kaydara FBX Binary  \x00\x1a\x00";

/// Header read error.
#[derive(Debug, Error)]
pub enum HeaderError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("FBX magic binary is not detected")]
    MagicNotDetected,
}

/// FBX binary header.
///
/// This type represents a binary header for all supported versions of FBX.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FbxHeader {
    /// FBX version.
    version: FbxVersion,
}

impl FbxHeader {
    /// Reads an FBX header from the given reader.
    pub async fn load(mut reader: impl AsyncRead + Unpin) -> Result<Self, HeaderError> {
        // Check magic.
        let mut magic_buf = [0u8; MAGIC_LEN];
        reader.read_exact(&mut magic_buf).await?;
        if magic_buf != *MAGIC {
            return Err(HeaderError::MagicNotDetected);
        }

        // Read FBX version.
        let version = reader.read_u32::<LE>().await?;
        info!("FBX header is detected, version={}", version);

        Ok(FbxHeader {
            version: FbxVersion::new(version),
        })
    }

    /// Returns FBX version.
    pub fn version(self) -> FbxVersion {
        self.version
    }

    /// Returns header length in bytes.
    pub fn len(self) -> usize {
        /// FBX version length.
        const VERSION_LEN: usize = 4;

        MAGIC_LEN + VERSION_LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_lite::io::Cursor;

    #[async_std::test]
    async fn header_ok() {
        let raw_header = b"Kaydara FBX Binary  \x00\x1a\x00\xe8\x1c\x00\x00";
        let mut cursor = Cursor::new(raw_header);
        let header = FbxHeader::load(&mut cursor)
            .await
            .expect("Should never fail");
        assert_eq!(
            header.version(),
            FbxVersion::new(7400),
            "Header and version should be detected correctly"
        );
        assert_eq!(
            cursor.position() as usize,
            raw_header.len(),
            "Header should be read completely"
        );
    }

    #[async_std::test]
    async fn magic_ng() {
        let wrong_header = b"Kaydara FBX Binary  \x00\xff\x00\xe8\x1c\x00\x00";
        let mut cursor = Cursor::new(wrong_header);
        // `HeaderError` may contain `io::Error` and is not comparable.
        assert!(
            matches!(
                FbxHeader::load(&mut cursor).await,
                Err(HeaderError::MagicNotDetected)
            ),
            "Invalid magic should be reported by `MagicNotDetected`"
        );
        assert!(
            (cursor.position() as usize) < wrong_header.len(),
            "Header should not be read too much if the magic is not detected"
        );
    }
}
