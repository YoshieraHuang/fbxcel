use std::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use crate::ready_ok;
use byteorder::ByteOrder;
use futures_util::{AsyncRead, Future};
use pin_project_lite::pin_project;
use std::io::{ErrorKind, Result};

macro_rules! reader {
    ($name:ident, $ty:ty, $reader:ident) => {
        reader!($name, $ty, $reader, core::mem::size_of::<$ty>());
    };
    ($name:ident, $ty:ty, $reader:ident, $bytes:expr) => {
        pin_project! {
            #[doc(hidden)]
            #[must_use = "futures do nothing unless you `.await` or poll them"]
            pub struct $name<R, BO>
            {
                #[pin]
                reader: R,
                buffer: [u8; $bytes],
                read: u8,
                _byte_order: PhantomData<BO>,
            }
        }

        impl<R, BO: ByteOrder> $name<R, BO> {
            pub(crate) fn new(reader: R) -> $name<R, BO> {
                $name {
                    reader,
                    buffer: [0u8; $bytes],
                    read: 0,
                    _byte_order: PhantomData,
                }
            }
        }

        impl<R, BO> Future for $name<R, BO>
        where
            R: AsyncRead + Unpin,
            BO: ByteOrder,
        {
            type Output = Result<$ty>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let mut this = self.project();

                if *this.read == $bytes as u8 {
                    return Poll::Ready(Ok(BO::$reader(this.buffer)));
                }

                while *this.read < $bytes as u8 {
                    let mut buf = &mut this.buffer[*this.read as usize..];
                    let n = ready_ok!(this.reader.as_mut().poll_read(cx, &mut buf));
                    if n == 0 {
                        return Poll::Ready(Err(ErrorKind::UnexpectedEof.into()));
                    }

                    *this.read += n as u8;
                }

                let num = BO::$reader(this.buffer);
                // clear the read number and ready for next read
                *this.read = 0;

                Poll::Ready(Ok(num))
            }
        }
    };
}

macro_rules! reader8 {
    ($name:ident, $ty:ty) => {
        pin_project! {
            /// Future returned from `read_u8`
            #[doc(hidden)]
            #[must_use = "futures do nothing unless you `.await` or poll them"]
            pub struct $name<R> {
                #[pin]
                reader: R,
            }
        }

        impl<R> $name<R> {
            pub(crate) fn new(reader: R) -> $name<R> {
                $name { reader }
            }
        }

        impl<R> Future for $name<R>
        where
            R: AsyncRead + Unpin,
        {
            type Output = Result<$ty>;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = self.project();

                let mut buf = [0; 1];
                let n = ready_ok!(this.reader.poll_read(cx, &mut buf));
                if n == 0 {
                    return Poll::Ready(Err(ErrorKind::UnexpectedEof.into()));
                }

                Poll::Ready(Ok(buf[0] as $ty))
            }
        }
    };
}

reader8!(ReadU8, u8);
reader8!(ReadI8, i8);

reader!(ReadU16, u16, read_u16);
reader!(ReadI16, i16, read_i16);
reader!(ReadU32, u32, read_u32);
reader!(ReadI32, i32, read_i32);
reader!(ReadU64, u64, read_u64);
reader!(ReadI64, i64, read_i64);
reader!(ReadI128, i128, read_i128);
reader!(ReadF32, f32, read_f32);
reader!(ReadF64, f64, read_f64);
