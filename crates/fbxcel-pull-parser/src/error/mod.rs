//! Errors and result types.
//!
//! Types in this module will be used among multiple versions of parsers.

use fbxcel_low::LowError;
use std::{error, io};
use thiserror::Error;

use crate::SyntacticPosition;

pub use self::{
    data::{Compression, DataError},
    operation::OperationError,
    warning::Warning,
};

mod data;
mod operation;
mod warning;

/// Parsing result.
pub type Result<T> = std::result::Result<T, Error>;

/// Parsing error.
#[derive(Debug, Error)]
pub struct Error(Box<Repr>);

impl Error {
    /// Returns the error kind.
    pub fn kind(&self) -> ErrorKind {
        self.0.error.kind()
    }

    /// Returns a reference to the inner error container.
    pub fn get_ref(&self) -> &ErrorContainer {
        &self.0.error
    }

    /// Returns a reference to the inner error if the type matches.
    pub fn downcast_ref<T: 'static + error::Error>(&self) -> Option<&T> {
        self.0.error.as_error().downcast_ref::<T>()
    }

    /// Returns the syntactic position if available.
    pub fn position(&self) -> Option<&SyntacticPosition> {
        self.0.position.as_ref()
    }

    /// Creates a new `Error` with the given syntactic position info.
    pub(crate) fn with_position(error: ErrorContainer, position: SyntacticPosition) -> Self {
        Self(Box::new(Repr::with_position(error, position)))
    }

    /// Sets the syntactic position and returns the new error.
    pub(crate) fn and_position(mut self, position: SyntacticPosition) -> Self {
        self.0.position = Some(position);
        self
    }
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.error.fmt(f)
    }
}

impl<T: Into<ErrorContainer>> From<T> for Error {
    fn from(t: T) -> Self {
        Error(Box::new(Repr::new(t.into())))
    }
}

/// Internal representation of parsing error.
#[derive(Debug)]
struct Repr {
    /// Error.
    error: ErrorContainer,
    /// Syntactic position.
    position: Option<SyntacticPosition>,
}

impl Repr {
    /// Creates a new `Repr`.
    pub(crate) fn new(error: ErrorContainer) -> Self {
        Self {
            error,
            position: None,
        }
    }

    /// Creates a new `Repr` with the given syntactic position info.
    pub(crate) fn with_position(error: ErrorContainer, position: SyntacticPosition) -> Self {
        Self {
            error,
            position: Some(position),
        }
    }
}

/// Error kind for parsing errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// Invalid data.
    ///
    /// With this error kind, the inner error must be [`DataError`].
    ///
    /// [`DataError`]: enum.DataError.html
    Data,
    /// I/O error.
    ///
    /// With this error kind, the inner error must be [`std::io::Error`].
    ///
    /// [`std::io::Error`]:
    /// https://doc.rust-lang.org/stable/std/io/struct.Error.html
    Io,
    /// Invalid operation.
    ///
    /// With this error kind, the inner error must be [`OperationError`].
    ///
    /// [`OperationError`]: enum.OperationError.html
    Operation,
    /// Critical warning.
    ///
    /// With this error kind, the inner error must be [`Warning`].
    ///
    /// [`Warning`]: enum.Warning.html
    Warning,
}

/// Parsing error container.
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum ErrorContainer {
    #[error(transparent)]
    Data(#[from] DataError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Operation(#[from] OperationError),
    #[error(transparent)]
    Warning(#[from] Warning),
}

impl ErrorContainer {
    /// Returns the error kind of the error.
    pub fn kind(&self) -> ErrorKind {
        match self {
            ErrorContainer::Data(_) => ErrorKind::Data,
            ErrorContainer::Io(_) => ErrorKind::Io,
            ErrorContainer::Operation(_) => ErrorKind::Operation,
            ErrorContainer::Warning(_) => ErrorKind::Warning,
        }
    }

    /// Returns `&dyn std::error::Error`.
    pub fn as_error(&self) -> &(dyn 'static + error::Error) {
        match self {
            ErrorContainer::Data(e) => e,
            ErrorContainer::Io(e) => e,
            ErrorContainer::Operation(e) => e,
            ErrorContainer::Warning(e) => e,
        }
    }
}

impl From<LowError> for ErrorContainer {
    fn from(e: LowError) -> Self {
        ErrorContainer::Data(e.into())
    }
}
