//! Low-level or primitive data types for FBX 7.4 and compatible versions.

pub use self::{
    array_attribute::ArrayAttributeEncoding,
    attribute::{types::AttributeType, value::AttributeValue},
    fbx_footer::FbxFooter,
};
pub use self::{
    array_attribute::ArrayAttributeHeader, node_header::NodeHeader,
    special_attribute::SpecialAttributeHeader,
};

mod array_attribute;
mod attribute;
mod fbx_footer;
mod node_header;
mod special_attribute;
