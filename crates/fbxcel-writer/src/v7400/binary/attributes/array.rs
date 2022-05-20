//! Array attributes things.

use std::convert::TryFrom;

use crate::v7400::binary::{
    attributes::IntoBytes, stream_position::StreamPosition, AttributesWriter, Error, Result,
};
use async_compression::futures::write::ZlibEncoder;
use fbxcel_low::v7400::{ArrayAttributeEncoding, ArrayAttributeHeader, AttributeType};
use futures_lite::{io, AsyncSeek, AsyncWrite, AsyncWriteExt};

// /// A trait for types which can be represented as multiple bytes array.
// pub(crate) trait IntoBytesMulti<E>: Sized {
//     /// Calls the given function with the bytes array multiple times.
//     fn call_with_le_bytes_multi(
//         self,
//         f: impl FnMut(&[u8]) -> std::result::Result<(), E>,
//     ) -> std::result::Result<usize, E>;
// }

// impl<T: IntoBytes, E, I: IntoIterator<Item = std::result::Result<T, E>>> IntoBytesMulti<E> for I {
//     fn call_with_le_bytes_multi(
//         self,
//         mut f: impl FnMut(&[u8]) -> std::result::Result<(), E>,
//     ) -> std::result::Result<usize, E> {
//         let mut count = 0usize;
//         self.into_iter()
//             .inspect(|_| count = count.checked_add(1).expect("Too many elements"))
//             .try_for_each(|elem| elem?.call_with_le_bytes(&mut f))?;

//         Ok(count)
//     }
// }

/// Writes array elements into the given writer.
pub(crate) async fn write_elements_result_iter<T, E>(
    mut writer: impl AsyncWrite + Unpin,
    iter: impl IntoIterator<Item = std::result::Result<T, E>>,
) -> Result<u32>
where
    T: IntoBytes,
    T::Bytes: AsRef<[u8]>,
    Error: From<E>,
{
    let mut count = 0usize;
    for elem in iter.into_iter() {
        let bytes = elem?.into_bytes();
        writer.write_all(bytes.as_ref()).await?;
        count += 1;
    }
    let count =
        u32::try_from(count).map_err(|_| Error::TooManyArrayAttributeElements(count + 1))?;

    Ok(count)
}

/// Writes the given array attribute header.
pub(crate) async fn write_array_header(
    mut writer: impl AsyncWrite + Unpin,
    header: &ArrayAttributeHeader,
) -> io::Result<()> {
    writer
        .write_all(&header.elements_count.to_le_bytes())
        .await?;
    writer
        .write_all(&header.encoding.to_u32().to_le_bytes())
        .await?;
    writer.write_all(&header.bytelen.to_le_bytes()).await?;

    Ok(())
}

/// Writes the given array attribute.
pub(crate) async fn write_array_attr_result_iter<W, T, E>(
    writer: &mut AttributesWriter<'_, W>,
    ty: AttributeType,
    encoding: Option<ArrayAttributeEncoding>,
    iter: impl IntoIterator<Item = std::result::Result<T, E>>,
) -> Result<()>
where
    W: AsyncWrite + AsyncSeek + Unpin,
    T: IntoBytes,
    T::Bytes: AsRef<[u8]>,
    Error: From<E>,
{
    let encoding = encoding.unwrap_or(ArrayAttributeEncoding::Direct);

    let header_pos = writer.initialize_array(ty, encoding).await?;

    // Write elements.
    let start_pos = writer.sink().stream_position().await?;
    let elements_count = match encoding {
        ArrayAttributeEncoding::Direct => write_elements_result_iter(writer.sink(), iter).await?,
        ArrayAttributeEncoding::Zlib => {
            let mut sink = ZlibEncoder::new(writer.sink());
            write_elements_result_iter(&mut sink, iter).await?
        }
    };
    let end_pos = writer.sink().stream_position().await?;
    let bytelen = end_pos - start_pos;

    // Calculate header fields.
    let bytelen = u32::try_from(bytelen).map_err(|_| Error::AttributeTooLong(bytelen as usize))?;

    // Write real array header.
    writer
        .finalize_array(
            header_pos,
            &ArrayAttributeHeader {
                elements_count,
                encoding,
                bytelen,
            },
        )
        .await?;

    Ok(())
}
