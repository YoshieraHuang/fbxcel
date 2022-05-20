use async_trait::async_trait;
use futures_lite::{io, AsyncRead, AsyncSeek};

mod position_reader;
pub use position_reader::SeekableReader;

/// Asynchronous reading with known position
#[async_trait]
pub trait AsyncPositionRead: AsyncRead + AsyncSeek + Sized {
    /// Returns the offset of a byte which would be read next.
    fn position(&self) -> u64;

    /// Skips (seeks forward) the given size.
    ///
    ///
    async fn skip_distance(&mut self, distance: u64) -> io::Result<()>
    where
        Self: Unpin;

    /// Skips (seeks forward) to the given position.
    async fn skip_to(&mut self, pos: u64) -> io::Result<()>
    where
        Self: Unpin,
    {
        let distance = pos
            .checked_sub(self.position())
            .expect("Attemp to skip backward");
        self.skip_distance(distance).await
    }
}

#[async_trait]
impl<R: AsyncPositionRead + Unpin + Send> AsyncPositionRead for &mut R {
    fn position(&self) -> u64 {
        (**self).position()
    }

    async fn skip_distance(&mut self, distance: u64) -> io::Result<()>
    where
        Self: Unpin,
    {
        (**self).skip_distance(distance).await
    }

    async fn skip_to(&mut self, pos: u64) -> io::Result<()>
    where
        Self: Unpin,
    {
        (**self).skip_to(pos).await
    }
}
