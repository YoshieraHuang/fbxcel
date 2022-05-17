use thiserror::Error;

#[derive(Debug, Error)]
pub enum LowError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid array attribute encoding: {0}")]
    InvalidArrayAttributeEncoding(u32),
    #[error("invalid attribute type code: {0}")]
    InvalidAttributeTypeCode(u8),
}
