//! Reader functions and traits.

use async_trait::async_trait;
use futures_lite::io::AsyncRead;

use byte_order_reader::AsyncByteOrderRead;

use crate::pull_parser::{v7400::Parser, ParserSource, Result};

// /// A trait for types readable from a parser.
// pub(crate) trait FromParser: Sized {
//     /// Reads the data from the given parser.
//     fn read_from_parser<R: ParserSource>(parser: &mut Parser<R>) -> Result<Self>;
// }

// impl<T: FromReader> FromParser for T {
//     fn read_from_parser<R: ParserSource>(parser: &mut Parser<R>) -> Result<Self> {
//         FromReader::from_reader(parser.reader())
//     }
// }
