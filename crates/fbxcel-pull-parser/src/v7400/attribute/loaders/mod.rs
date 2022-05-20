//! Node attribute loaders.

pub use self::{
    direct::DirectLoader,
    single::{ArrayLoader, BinaryLoader, PrimitiveLoader, StringLoader},
    types::TypeLoader,
};

mod direct;
mod single;
mod types;
