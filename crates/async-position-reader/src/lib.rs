use async_trait::async_trait;
use futures_lite::{io, AsyncRead, AsyncReadExt};

mod inner_reader;
pub use inner_reader::InnerAsyncPositionReader;

mod position_reader;
pub use position_reader::{SeekableReader, SimpleReader};

/// Asynchronous reading with known position
#[async_trait]
pub trait AsyncPositionRead: AsyncRead + Sized {
    /// Returns the offset of a byte which would be read next.
    fn position(&self) -> u64;

    /// Skips (seeks forward) the given size.
    ///
    ///
    async fn skip_distance(&mut self, distance: u64) -> io::Result<()>
    where
        Self: Unpin,
    {
        let mut limited = self.take(distance);
        io::copy(&mut limited, &mut io::sink()).await?;
        Ok(())
    }

    /// Skips (seeks forward) to the given position.
    async fn skip_to(&mut self, pos: u64) -> io::Result<()>
    where
        Self: Unpin,
    {
        let distance = pos
            .checked_add(self.position())
            .expect("Attempt to skip backward");
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
