//! Node header.

use async_trait::async_trait;
use byte_order_reader::FromAsyncReader;
use futures_lite::AsyncRead;
use std::io::{Error, Result};

/// Node header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHeader {
    /// End offset of the node.
    pub end_offset: u64,
    /// The number of the node attributes.
    pub num_attributes: u64,
    /// Length of the node attributes in bytes.
    pub bytelen_attributes: u64,
    /// Length of the node name in bytes.
    pub bytelen_name: u8,
}

impl NodeHeader {
    /// Checks whether the entry indicates end of a node.
    pub fn is_node_end(&self) -> bool {
        self.end_offset == 0
            && self.num_attributes == 0
            && self.bytelen_attributes == 0
            && self.bytelen_name == 0
    }

    /// Returns node end marker.
    #[cfg(feature = "writer")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
    pub fn node_end() -> Self {
        Self {
            end_offset: 0,
            num_attributes: 0,
            bytelen_attributes: 0,
            bytelen_name: 0,
        }
    }
}

#[async_trait]
impl<R> FromAsyncReader<R> for NodeHeader
where
    R: AsyncRead + Unpin + Send,
{
    type Error = Error;

    async fn from_async_reader(reader: &mut R) -> Result<Self> {
        let end_offset = u32::from_async_reader(reader).await? as u64;
        let num_attributes = u32::from_async_reader(reader).await? as u64;
        let bytelen_attributes = u32::from_async_reader(reader).await? as u64;
        let bytelen_name = u8::from_async_reader(reader).await?;
        let inner = NodeHeader {
            end_offset,
            num_attributes,
            bytelen_attributes,
            bytelen_name,
        };
        Ok(inner)
    }
}
