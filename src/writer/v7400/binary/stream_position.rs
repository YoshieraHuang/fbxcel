use std::io::SeekFrom;

use futures_lite::{io::SeekFuture, AsyncSeek, AsyncSeekExt};

/// This method exists in `futures::AsyncSeekExt` but not in `futures-lite::AsyncSeekExt`.
/// Have to implement it by myself here.
pub(crate) trait StreamPosition: AsyncSeek {
    fn stream_position(&mut self) -> SeekFuture<Self>
    where
        Self: Unpin,
    {
        self.seek(SeekFrom::Current(0))
    }
}

impl<S: AsyncSeek + Unpin> StreamPosition for S {}
