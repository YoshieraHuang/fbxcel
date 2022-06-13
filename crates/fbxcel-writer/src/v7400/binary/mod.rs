//! Binary writer for FBX 7.4 or later.
//!
//! # Using writer
//!
//! ## Setup and finalize
//!
//! To setup writer, use [`Writer::new`].
//!
//! To finalize writer, use [`Writer::finalize`] or
//! [`Writer::finalize_and_flush`].
//! Users should explicitly finalize the writer, because finalizing is not
//! implicitly done on drop.
//!
//! ```
//! use fbxcel::{low::FbxVersion, writer::v7400::binary::{FbxFooter, Writer}};
//! # let mut sink = std::io::Cursor::new(Vec::new());
//! let mut writer = Writer::new(sink, FbxVersion::V7_4)?;
//!
//! // Do something here.
//!
//! // Prepare FBX footer.
//! // Use default if you don't care.
//! let footer = FbxFooter::default();
//! writer.finalize(&footer)?;
//! // Or `writer.finalize_and_flush(&footer)?;` if you want to flush.
//! # Ok::<_, fbxcel::writer::v7400::binary::Error>(())
//! ```
//!
//! ## Create node and add node attributes
//!
//! To create node, use [`Writer::new_node`].
//! It returns [`AttributesWriter`] and users can add node attributes to the
//! newly created node through it.
//!
//! Once `AttributesWriter` is dropped, you cannot add node attributes to the
//! node anymore.
//!
//! ```
//! use fbxcel::{
//!     low::{v7400::ArrayAttributeEncoding, FbxVersion},
//!     writer::v7400::binary::Writer,
//! };
//! # let mut sink = std::io::Cursor::new(Vec::new());
//! let mut writer = Writer::new(sink, FbxVersion::V7_4)?;
//!
//! // Create a node with name `NodeName`.
//! let mut attrs_writer = writer.new_node("NodeName")?;
//!
//! // Add attributes to the node.
//! attrs_writer.append_bool(true)?;
//! // If you don't care about compression, pass `None`.
//! attrs_writer.append_arr_i32_from_iter(None, [1, 2, 4, 8, 16].iter().cloned())?;
//! // If you want to use specific compression, pass `Some(_)`.
//! attrs_writer.append_arr_f32_from_iter(
//!     Some(ArrayAttributeEncoding::Zlib),
//!     [3.14, 1.412].iter().cloned(),
//! )?;
//! attrs_writer.append_string_direct("Hello, world")?;
//!
//! # Ok::<_, fbxcel::writer::v7400::binary::Error>(())
//! ```
//!
//! ## Close current node
//!
//! Simply call [`Writer::close_node`].
//!
//! It is user's responsibility to manage depth of current node and avoid
//! calling extra `close_node`.
//!
//! If `close_node` call is too few and there remains open nodes on finalizing
//! writer, `finalize()` and `finalize_and_flush()` will return error.
//!
//! ```
//! use fbxcel::{
//!     low::{v7400::ArrayAttributeEncoding, FbxVersion},
//!     writer::v7400::binary::Writer,
//! };
//! # let mut sink = std::io::Cursor::new(Vec::new());
//! let mut writer = Writer::new(sink, FbxVersion::V7_4)?;
//!
//! // Create a node with name `NodeName`.
//! let mut attrs_writer = writer.new_node("NodeName")?;
//!
//! // Do something here.
//! # let _ = &attrs_writer;
//!
//! // To close current node, simply call `close_node()`.
//! writer.close_node()?;
//!
//! # Ok::<_, fbxcel::writer::v7400::binary::Error>(())
//! ```

use std::{convert::TryFrom, io::SeekFrom};

use futures_util::{io, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use log::{debug, trace};

use fbxcel_low::{v7400::NodeHeader, FbxVersion, MAGIC};

pub use self::{
    attributes::AttributesWriter,
    error::{CompressionError, Error, Result},
    footer::{FbxFooter, FbxFooterPaddingLength},
};

mod macros;

mod attributes;
mod error;
mod footer;
mod stream_position;
use stream_position::StreamPosition;

/// Binary writer.
///
/// See [module documentation][`self`] for usage.
#[derive(Debug, Clone)]
pub struct Writer<W> {
    /// Writer destination.
    sink: W,
    /// FBX version.
    fbx_version: FbxVersion,
    /// Node header positions not yet closed.
    open_nodes: Vec<OpenNode>,
}

impl<W> Writer<W> {
    /// Creates a new `Writer` and writes FBX file header.
    pub async fn new(mut sink: W, fbx_version: FbxVersion) -> Result<Self>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        // Check if the given version is supported.
        if fbx_version.major() != 7 {
            return Err(Error::UnsupportedFbxVersion(fbx_version));
        }

        // Write FBX magic binary.
        sink.seek(SeekFrom::Start(0)).await?;
        sink.write_all(MAGIC).await?;
        sink.write_all(&fbx_version.raw().to_le_bytes()).await?;

        Ok(Self {
            sink,
            fbx_version,
            open_nodes: Vec::new(),
        })
    }

    /// Returns a mutable reference to the sink.
    fn sink(&mut self) -> &mut W {
        &mut self.sink
    }

    /// Returns a mutable reference to the node header of the current node.
    fn current_node(&mut self) -> Option<&mut OpenNode> {
        self.open_nodes.last_mut()
    }

    /// Returns a mutable reference to the node header of the current node.
    fn current_node_header(&mut self) -> Option<&mut NodeHeader> {
        self.current_node().map(|v| &mut v.header)
    }

    /// Writes the given node header.
    async fn write_node_header(&mut self, header: &NodeHeader) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        if self.fbx_version.raw() < 7500 {
            self.sink
                .write_all(
                    &u32::try_from(header.end_offset)
                        .map_err(|_| Error::FileTooLarge(header.end_offset))?
                        .to_le_bytes(),
                )
                .await?;
            self.sink
                .write_all(
                    &u32::try_from(header.num_attributes)
                        .map_err(|_| Error::TooManyAttributes(header.num_attributes as usize))?
                        .to_le_bytes(),
                )
                .await?;
            self.sink
                .write_all(
                    &u32::try_from(header.bytelen_attributes)
                        .map_err(|_| Error::AttributeTooLong(header.bytelen_attributes as usize))?
                        .to_le_bytes(),
                )
                .await?;
        } else {
            self.sink
                .write_all(&header.end_offset.to_le_bytes())
                .await?;
            self.sink
                .write_all(&header.num_attributes.to_le_bytes())
                .await?;
            self.sink
                .write_all(&header.bytelen_attributes.to_le_bytes())
                .await?;
        }
        self.sink
            .write_all(&header.bytelen_name.to_le_bytes())
            .await?;
        Ok(())
    }

    /// Writes the FBX version.
    async fn write_fbx_verison(&mut self) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        self.sink
            .write_all(&self.fbx_version.raw().to_le_bytes())
            .await
            .map_err(Into::into)
    }

    /// Finalizes node attributes and update node header info.
    async fn finalize_attributes(&mut self) -> Result<()>
    where
        W: AsyncSeek + Unpin,
    {
        trace!("Finalizing attributes: depth={:?}", self.open_nodes.len());

        let current_node = match self.open_nodes.last_mut() {
            Some(v) => v,
            None => {
                trace!("`finalize_attributes()` is called for root node, ignoring");
                return Ok(());
            }
        };
        if current_node.is_attrs_finalized {
            trace!("Attributes are already finalized");
            return Ok(());
        }

        let current_pos = self.sink.stream_position().await?;
        current_node.header.bytelen_attributes = current_pos - current_node.body_pos;
        current_node.is_attrs_finalized = true;

        trace!("Finalized attributes: current_node={:?}", current_node);

        Ok(())
    }

    /// Creates a new node and returns node attributes writer.
    pub async fn new_node(&mut self, name: &str) -> Result<AttributesWriter<'_, W>>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        trace!(
            "New node: name={:?}, depth={:?}",
            name,
            self.open_nodes.len()
        );
        self.finalize_attributes().await?;

        if let Some(current_node) = self.current_node() {
            current_node.has_child = true;
        }

        // Check if the node name is short enough.
        let bytelen_name =
            u8::try_from(name.len()).map_err(|_| Error::NodeNameTooLong(name.len()))?;

        let header_pos = self.sink.stream_position().await?;

        let header = NodeHeader {
            end_offset: 0,
            num_attributes: 0,
            bytelen_attributes: 0,
            bytelen_name,
        };

        // Write dummy header (placeholder).
        self.write_node_header(&header).await?;

        // Write node name.
        self.sink.write_all(name.as_ref()).await?;

        let body_pos = self.sink.stream_position().await?;

        self.open_nodes.push(OpenNode {
            header_pos,
            body_pos,
            header,
            has_child: false,
            is_attrs_finalized: false,
        });

        Ok(AttributesWriter::new(self))
    }

    /// Closes an open node.
    pub async fn close_node(&mut self) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        trace!("Close node: depth={:?}", self.open_nodes.len());
        self.finalize_attributes().await?;

        let mut current_node = match self.open_nodes.pop() {
            Some(v) => v,
            None => return Err(Error::NoNodesToClose),
        };

        // Write node end marker if necessary.
        if current_node.has_child || current_node.header.num_attributes == 0 {
            self.write_node_header(&NodeHeader::node_end()).await?;
        }

        // Update node header.
        let node_end_pos = self.sink.stream_position().await?;
        self.sink
            .seek(SeekFrom::Start(current_node.header_pos))
            .await?;
        current_node.header.end_offset = node_end_pos;
        assert_eq!(
            current_node.header.num_attributes == 0,
            current_node.header.bytelen_attributes == 0,
            "Length of attributes can be zero iff there are no attributes"
        );
        self.write_node_header(&current_node.header).await?;
        self.sink.seek(SeekFrom::Start(node_end_pos)).await?;

        Ok(())
    }

    /// Writes the given tree.
    #[cfg(feature = "tree")]
    #[cfg_attr(feature = "docsrs", doc(cfg(feature = "tree")))]
    pub async fn write_tree(&mut self, tree: fbxcel_tree::v7400::Tree) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        use fbxcel_low::v7400::AttributeValue;

        let mut current = match tree.root().first_child() {
            Some(v) => v,
            None => return Ok(()),
        };

        'all: loop {
            let mut attrs_writer = self.new_node(current.name()).await?;
            for attr in current.attributes() {
                match attr {
                    AttributeValue::Bool(v) => attrs_writer.append_bool(*v).await?,
                    AttributeValue::I16(v) => attrs_writer.append_i16(*v).await?,
                    AttributeValue::I32(v) => attrs_writer.append_i32(*v).await?,
                    AttributeValue::I64(v) => attrs_writer.append_i64(*v).await?,
                    AttributeValue::F32(v) => attrs_writer.append_f32(*v).await?,
                    AttributeValue::F64(v) => attrs_writer.append_f64(*v).await?,
                    AttributeValue::ArrBool(v) => {
                        attrs_writer
                            .append_arr_bool_from_iter(None, v.iter().cloned())
                            .await?
                    }
                    AttributeValue::ArrI32(v) => {
                        attrs_writer
                            .append_arr_i32_from_iter(None, v.iter().cloned())
                            .await?
                    }
                    AttributeValue::ArrI64(v) => {
                        attrs_writer
                            .append_arr_i64_from_iter(None, v.iter().cloned())
                            .await?
                    }
                    AttributeValue::ArrF32(v) => {
                        attrs_writer
                            .append_arr_f32_from_iter(None, v.iter().cloned())
                            .await?
                    }
                    AttributeValue::ArrF64(v) => {
                        attrs_writer
                            .append_arr_f64_from_iter(None, v.iter().cloned())
                            .await?
                    }
                    AttributeValue::Binary(v) => attrs_writer.append_binary_direct(v).await?,
                    AttributeValue::String(v) => attrs_writer.append_string_direct(v).await?,
                }
            }

            let mut visit_child = true;
            current = 'next: loop {
                if visit_child {
                    if let Some(child) = current.first_child() {
                        break 'next child;
                    }
                    // No children.
                    visit_child = false;
                }
                self.close_node().await?;
                if let Some(sib) = current.next_sibling() {
                    break 'next sib;
                }
                let parent = current
                    .parent()
                    .expect("Should never fail: `current` must not be the root note");
                if parent.node_id() == tree.root().node_id() {
                    break 'all;
                }
                current = parent;
            };
        }

        Ok(())
    }

    /// Finalizes the FBX binary and returns the inner sink.
    ///
    /// You may want to use [`finalize_and_flush()`][`Self::finalize_and_flush()`].
    pub async fn finalize(mut self, footer: &FbxFooter<'_>) -> Result<W>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        self.finalize_impl(footer).await?;

        Ok(self.sink)
    }

    /// Finalizes the FBX binary, and returns the inner sink after flushing.
    pub async fn finalize_and_flush(mut self, footer: &FbxFooter<'_>) -> Result<W>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        self.finalize_impl(footer).await?;
        self.sink.flush().await?;

        Ok(self.sink)
    }

    /// Internal implementation of `finalize()` and `finalize_and_flush()`.
    async fn finalize_impl(&mut self, footer: &FbxFooter<'_>) -> Result<()>
    where
        W: AsyncWrite + AsyncSeek + Unpin,
    {
        if !self.open_nodes.is_empty() {
            return Err(Error::UnclosedNode(self.open_nodes.len()));
        }

        // Close implicit root node.
        self.write_node_header(&NodeHeader::node_end()).await?;

        // Write FBX footer.
        self.sink.write_all(footer.unknown1()).await?;
        {
            let len = match footer.padding_len {
                FbxFooterPaddingLength::Default => {
                    let current = self.sink.stream_position().await?;
                    current.wrapping_neg() & 0x0f
                }
                FbxFooterPaddingLength::Forced(len) => u64::from(len),
            };
            debug!(
                "Footer padding: spec={:?}, len={:?}",
                footer.padding_len, len
            );
            io::copy(&mut io::repeat(0).take(len), &mut self.sink).await?;
        }
        self.sink.write_all(&footer.unknown2()).await?;
        self.write_fbx_verison().await?;
        io::copy(&mut io::repeat(0).take(120), &mut self.sink).await?;
        self.sink.write_all(footer.unknown3()).await?;

        Ok(())
    }
}

/// Open node state.
#[derive(Debug, Clone, Copy)]
struct OpenNode {
    /// Header position.
    header_pos: u64,
    /// Position of beginning of attributes part.
    body_pos: u64,
    /// Header.
    header: NodeHeader,
    /// Whether the node has child.
    has_child: bool,
    /// Whether the attributes are finalized.
    is_attrs_finalized: bool,
}
