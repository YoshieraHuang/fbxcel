//! FBX 7.4 footer.

use crate::FbxVersion;

/// FBX 7.4 footer.
///
/// Data contained in a FBX 7.4 footer is not useful for normal usage.
/// Most of users can safely ignore the footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FbxFooter {
    /// Unknown (semirandom) 16-bytes data.
    ///
    /// This field is expected to have prescribed upper 4 bits, i.e. the field
    /// is `fx bx ax 0x dx cx dx 6x bx 7x fx 8x 1x fx 2x 7x` if the FBX data is
    /// exported from official SDK.
    ///
    /// Note that third party exporter will use completely random data.
    pub unknown1: [u8; 16],
    /// Padding length.
    ///
    /// Padding is `padding_len` `0`s.
    /// `padding_len >= 0 && padding <= 15` should hold.
    ///
    /// Note that third party exporter will not use correct padding length.
    pub padding_len: u8,
    /// Unknown 4-bytes data.
    ///
    /// This is expected to be `[0u8; 4]`.
    pub unknown2: [u8; 4],
    /// FBX version.
    ///
    /// This is expected to be same as the version in header.
    pub fbx_version: FbxVersion,
    /// Unknown 16-bytes data.
    ///
    /// This is expected to be `[0xf8, 0x5a, 0x8c, 0x6a, 0xde, 0xf5, 0xd9, 0x7e,
    /// 0xec, 0xe9, 0x0c, 0xe3, 0x75, 0x8f, 0x29, 0x0b]`.
    pub unknown3: [u8; 16],
}
