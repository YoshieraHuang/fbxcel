use async_trait::async_trait;
use byteorder::{ByteOrder, LE};
use futures_lite::{io, AsyncRead};
use reader::{
    ReadF32, ReadF64, ReadI128, ReadI16, ReadI32, ReadI64, ReadI8, ReadU16, ReadU32, ReadU64,
    ReadU8,
};

mod reader;
mod util;

macro_rules! read_impl {
    (
        $(
            $(#[$outer:meta])*
            fn $name:ident(&mut self) -> $($fut:ident)*;
        )*
    ) => {
        $(
            $(#[$outer])*
            fn $name<BO>(&mut self) -> $($fut)*<&'_ mut Self, BO>
            where
                Self: Unpin,
                BO: ByteOrder,
            {
                $($fut)*::new(self)
            }
        )*
    };
}

pub trait AsyncByteOrderRead: Sized + AsyncRead {
    fn read_u8(&mut self) -> ReadU8<&mut Self>
    where
        Self: Unpin,
    {
        ReadU8::new(self)
    }

    fn read_i8(&mut self) -> ReadI8<&mut Self>
    where
        Self: Unpin,
    {
        ReadI8::new(self)
    }

    read_impl! {
        fn read_u16(&mut self) -> ReadU16;
        fn read_i16(&mut self) -> ReadI16;
        fn read_i32(&mut self) -> ReadI32;
        fn read_u32(&mut self) -> ReadU32;
        fn read_i64(&mut self) -> ReadI64;
        fn read_u64(&mut self) -> ReadU64;
        fn read_i128(&mut self) -> ReadI128;
        fn read_f32(&mut self) -> ReadF32;
        fn read_f64(&mut self) -> ReadF64;
    }
}

impl<T> AsyncByteOrderRead for T where T: AsyncRead {}

#[async_trait]
pub trait FromAsyncReader<R>: Sized
where
    R: io::AsyncRead + Unpin + Send,
{
    type Error: From<io::Error>;

    async fn from_async_reader(reader: &mut R) -> Result<Self, Self::Error>;
}

macro_rules! from_reader_impl {
    (
        $(
            ($ty:ty, $reader:ident)
        ),*
    ) => {
        $(
            #[async_trait]
            impl<R> FromAsyncReader<R> for $ty
            where
                R: io::AsyncRead + Unpin + Send
            {
                type Error = io::Error;

                async fn from_async_reader(reader: &mut R) -> io::Result<Self>
                where
                    R: 'async_trait
                {
                    $reader::<&mut R, LE>::new(reader).await
                }
            }
        )*
    }
}

#[async_trait]
impl<R> FromAsyncReader<R> for u8
where
    R: io::AsyncRead + Unpin + Send,
{
    type Error = io::Error;

    async fn from_async_reader(reader: &mut R) -> io::Result<Self> {
        ReadU8::new(reader).await
    }
}

#[async_trait]
impl<R> FromAsyncReader<R> for i8
where
    R: io::AsyncRead + Unpin + Send,
{
    type Error = io::Error;

    async fn from_async_reader(reader: &mut R) -> io::Result<Self> {
        ReadI8::new(reader).await
    }
}

from_reader_impl!(
    (u16, ReadU16),
    (i16, ReadI16),
    (u32, ReadU32),
    (i32, ReadI32),
    (u64, ReadU64),
    (i64, ReadI64),
    (i128, ReadI128),
    (f32, ReadF32),
    (f64, ReadF64)
);
