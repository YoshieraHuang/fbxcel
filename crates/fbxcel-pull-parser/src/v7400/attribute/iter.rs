//! Node attribute iterators.

use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use async_position_reader::AsyncPositionRead;
use futures_core::{ready, Stream};
use futures_lite::{AsyncBufRead, FutureExt};

use crate::{
    v7400::attribute::{loader::LoadAttribute, Attributes},
    Result,
};

/// Creates size hint from the given attributes and loaders.
fn make_size_hint_for_attrs<R, V>(
    attributes: &Attributes<'_, R>,
    loaders: &impl Iterator<Item = V>,
) -> (usize, Option<usize>)
where
    R: AsyncPositionRead,
    V: LoadAttribute,
{
    let (loaders_min, loaders_max) = loaders.size_hint();
    let attrs_rest = attributes.rest_count() as usize;
    let min = std::cmp::min(attrs_rest, loaders_min);
    let max = loaders_max.map_or_else(usize::max_value, |v| std::cmp::min(attrs_rest, v));

    (min, Some(max))
}

// /// Loads the next attrbute.
// async fn load_next<R, V>(
//     attributes: &mut Attributes<'_, R>,
//     loaders: &mut impl Stream<Item = V>,
//     cx: &mut Context,
// ) -> Option<Result<V::Output>>
// where
//     R: AsyncPositionRead + Unpin + Send,
//     V: LoadAttribute,
// {
//     let loader = loaders.next().await?;
//     attributes.load_next(loader).await.transpose()
// }

/// Node attributes iterator.
#[derive(Debug)]
pub struct BorrowedStream<'a, 'r, R, I> {
    /// Attributes.
    attributes: &'a mut Attributes<'r, R>,
    /// Loaders.
    loaders: I,
}

impl<'a, 'r, R, I, V> BorrowedStream<'a, 'r, R, I>
where
    R: AsyncPositionRead,
    I: Iterator<Item = V>,
    V: LoadAttribute,
{
    /// Creates a new iterator.
    pub(crate) fn new(attributes: &'a mut Attributes<'r, R>, loaders: I) -> Self {
        Self {
            attributes,
            loaders,
        }
    }
}

impl<'a, 'r, R, I, V> Stream for BorrowedStream<'a, 'r, R, I>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
    I: Iterator<Item = V> + Unpin,
    V: LoadAttribute + Send,
{
    type Item = Result<V::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let loader = this.loaders.next();
        Poll::Ready(if let Some(loader) = loader {
            ready!(this.attributes.load_next(loader).boxed().poll(cx)).transpose()
        } else {
            None
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        make_size_hint_for_attrs(self.attributes, &self.loaders)
    }
}

// impl<'a, 'r, R, I, V> FusedStream for BorrowedStream<'a, 'r, R, I>
// where
//     R: AsyncPositionRead + Unpin + Send,
//     I: Iterator<Item = V>,
//     V: LoadAttribute,
// {
// }

/// Node attributes iterator with buffered I/O.
#[derive(Debug)]
pub struct BorrowedStreamBuffered<'a, 'r, R, I> {
    /// Attributes.
    attributes: &'a mut Attributes<'r, R>,
    /// Loaders.
    loaders: I,
}

impl<'a, 'r, R, I, V> BorrowedStreamBuffered<'a, 'r, R, I>
where
    R: AsyncPositionRead,
    I: Iterator<Item = V>,
    V: LoadAttribute,
{
    /// Creates a new iterator.
    pub(crate) fn new(attributes: &'a mut Attributes<'r, R>, loaders: I) -> Self {
        Self {
            attributes,
            loaders,
        }
    }
}

impl<'a, 'r, R, I, V> Stream for BorrowedStreamBuffered<'a, 'r, R, I>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
    I: Iterator<Item = V> + Unpin,
    V: LoadAttribute + Send,
{
    type Item = Result<V::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let loader = this.loaders.next();
        Poll::Ready(if let Some(loader) = loader {
            ready!(this.attributes.load_next(loader).boxed().poll(cx)).transpose()
        } else {
            None
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        make_size_hint_for_attrs(self.attributes, &self.loaders)
    }
}

// impl<'a, 'r, R, I, V> FusedStream for BorrowedStreamBuffered<'a, 'r, R, I>
// where
//     R: AsyncPositionRead + io::BufRead,
//     I: Iterator<Item = V>,
//     V: LoadAttribute,
// {
// }

/// Node attributes iterator.
#[derive(Debug)]
pub struct OwnedIter<'r, R, I> {
    /// Attributes.
    attributes: Attributes<'r, R>,
    /// Loaders.
    loaders: I,
}

impl<'r, R, I, V> OwnedIter<'r, R, I>
where
    R: AsyncPositionRead,
    I: Iterator<Item = V>,
    V: LoadAttribute,
{
    /// Creates a new `Iter`.
    pub(crate) fn new(attributes: Attributes<'r, R>, loaders: I) -> Self {
        Self {
            attributes,
            loaders,
        }
    }
}

impl<'r, R, I, V> Stream for OwnedIter<'r, R, I>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
    I: Iterator<Item = V> + Unpin,
    V: LoadAttribute + Send,
{
    type Item = Result<V::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let loader = this.loaders.next();
        Poll::Ready(if let Some(loader) = loader {
            ready!(this.attributes.load_next(loader).boxed().poll(cx)).transpose()
        } else {
            None
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        make_size_hint_for_attrs(&self.attributes, &self.loaders)
    }
}

// impl<'r, R, I, V> iter::FusedIterator for OwnedIter<'r, R, I>
// where
//     R: AsyncPositionRead,
//     I: Iterator<Item = V>,
//     V: LoadAttribute,
// {
// }

/// Node attributes iterator with buffered I/O.
#[derive(Debug)]
pub struct OwnedIterBuffered<'r, R, I> {
    /// Attributes.
    attributes: Attributes<'r, R>,
    /// Loaders.
    loaders: I,
}

impl<'r, R, I, V> OwnedIterBuffered<'r, R, I>
where
    R: AsyncPositionRead,
    I: Iterator<Item = V>,
    V: LoadAttribute,
{
    /// Creates a new iterator.
    pub(crate) fn new(attributes: Attributes<'r, R>, loaders: I) -> Self {
        Self {
            attributes,
            loaders,
        }
    }
}

impl<'r, R, I, V> Stream for OwnedIterBuffered<'r, R, I>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
    I: Iterator<Item = V> + Unpin,
    V: LoadAttribute + Send,
{
    type Item = Result<V::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let loader = this.loaders.next();
        Poll::Ready(if let Some(loader) = loader {
            ready!(this.attributes.load_next(loader).boxed().poll(cx)).transpose()
        } else {
            None
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        make_size_hint_for_attrs(&self.attributes, &self.loaders)
    }
}

// impl<'r, R, I, V> FusedStream for OwnedIterBuffered<'r, R, I>
// where
//     R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
//     I: FusedIterator<Item = V>,
//     V: LoadAttribute,
// {
// }
