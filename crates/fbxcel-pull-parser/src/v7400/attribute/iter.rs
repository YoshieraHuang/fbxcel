//! Node attribute iterators.

use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use async_position_reader::AsyncPositionRead;
use futures_util::Stream;
use futures_util::{AsyncBufRead, FutureExt};
use pin_project_lite::pin_project;

use crate::{
    v7400::attribute::{loader::LoadAttribute, Attributes},
    Result,
};

/// Creates size hint from the given attributes and loaders.
fn make_size_hint_for_attrs<R, V>(
    attributes: &Attributes<R>,
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

// pin_project! {
//     /// Node attributes iterator.
//     pub struct BorrowedStream<'a, 'r, R, I, O> {
//         attributes: &'a mut Attributes<'r, R>,
//         loaders: I,
//         exhausted: bool,
//         #[pin]
//         fut: Option<BoxFuture<'a, Result<Option<O>>>>
//     }
// }

// impl<'a, 'r, R, I, V, O> BorrowedStream<'a, 'r, R, I, O>
// where
//     R: AsyncPositionRead,
//     I: Iterator<Item = V>,
//     V: LoadAttribute<Output = O>,
// {
//     /// Creates a new iterator.
//     pub(crate) fn new(attributes: &'a mut Attributes<'r, R>, loaders: I) -> Self {
//         Self {
//             attributes,
//             loaders,
//             exhausted: false,
//             fut: None,
//         }
//     }
// }

// impl<'a, 'r, R, I, V, O> Stream for BorrowedStream<'a, 'r, R, I, O>
// where
//     R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
//     I: Iterator<Item = V> + Unpin,
//     V: LoadAttribute<Output = O> + Send + 'a,
// {
//     type Item = Result<O>;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> 
//     {
//         let mut this = self.project();
//         let mut fut = match this.fut.take() {
//             Some(fut) => fut,
//             None => {
//                 match this.loaders.next() {
//                     None => {
//                         *this.exhausted = true;
//                         return Poll::Ready(None);
//                     }
//                     Some(loader) => {
//                         this.attributes.load_next(loader).boxed()
//                     }
//                 }
//             }
//         };
//         match fut.poll_unpin(cx) {
//             Poll::Pending => {
//                 this.fut.set(Some(fut));
//                 Poll::Pending
//             },
//             Poll::Ready(res) => {
//                 Poll::Ready(res.transpose())
//             }
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         make_size_hint_for_attrs(self.attributes, &self.loaders)
//     }
// }

// // impl<'a, 'r, R, I, V> FusedStream for BorrowedStream<'a, 'r, R, I>
// // where
// //     R: AsyncPositionRead + Unpin + Send,
// //     I: Iterator<Item = V>,
// //     V: LoadAttribute,
// // {
// // }

pin_project! {
    pub struct OwnedIter<'a, R, I> {
        attributes: Attributes<'a, R>,
        loaders: I,
        exhausted: bool,
        // #[pin]
        // fut: Option<BoxFuture<'a, Result<Option<O>>>>
    }
}

impl<'a, R, I, V> OwnedIter<'a, R, I>
where
    R: AsyncPositionRead,
    I: Iterator<Item = V>,
    V: LoadAttribute,
{
    /// Creates a new `Iter`.
    pub(crate) fn new(attributes: Attributes<'a, R>, loaders: I) -> Self {
        Self {
            attributes,
            loaders,
            exhausted: false,
            // fut: None,
        }
    }
}

impl<'a, R, I, V> Stream for OwnedIter<'a, R, I>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
    I: Iterator<Item = V> + Unpin,
    V: LoadAttribute + Send + 'a,
{
    type Item = Result<V::Output>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.loaders.next() {
            None => {
                *this.exhausted = true;
                return Poll::Ready(None);
            }
            Some(loader) => {
                let mut fut = this.attributes.load_next(loader).boxed();
                // Poll the future until ready to avoid temporary storage.
                loop {
                    if let Poll::Ready(res) = fut.poll_unpin(cx) {
                        return Poll::Ready(res.transpose());
                    }
                }
            }
        }
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
