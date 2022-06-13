//! Macros.

/// Drives the given writer to write the given tree as FBX binary, and returns
/// the `fbxcel::writer::v7400::binary::Result<()>`.
///
/// Enabled by `writer` feature.
///
/// # Examples
///
/// ```
/// # use fbxcel::write_v7400_binary;
/// use fbxcel::{low::FbxVersion, writer::v7400::binary::Writer};
/// let mut writer = Writer::new(std::io::Cursor::new(Vec::new()), FbxVersion::V7_4)?;
///
/// write_v7400_binary!(
///     writer=writer,
///     tree={
///         Node0: {
///             Node0_0: {}
///             Node0_1: {}
///         }
///         Node1: {
///             // You can use trailing comma.
///             Node1_0: {},
///             Node1_1: {},
///         }
///         // Use parens to specify attributes by single array.
///         // Note that the expression inside parens should implement
///         // `IntoIterator<Item = AttributeValue>`.
///         Node2: (vec!["hello".into(), "world".into(), 42i32.into()]) {}
///         // Use brackets to specify attributes one by one.
///         Node3: ["hello", "world", 1.234f32, &b"BINARY"[..]] {}
///     },
/// )?;
/// let _buf = writer.finalize_and_flush(&Default::default())?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
#[cfg_attr(feature = "docsrs", doc(cfg(feature = "writer")))]
#[macro_export]
macro_rules! write_v7400_binary {
    (
        writer=$writer:expr,
        tree={$($tree:tt)*},
    ) => {{
        let f = async {
            let _writer = &mut $writer;
            write_v7400_binary! { @__node, _writer, $($tree)* };
            std::result::Result::<_, $crate::v7400::binary::Error>::Ok(())
        };
        f.await
    }};


    (@__node, $writer:ident,) => {};
    (@__node, $writer:ident, , $($tree:tt)*) => {
        write_v7400_binary! { @__node, $writer, $($tree)* }
    };

    (@__node, $writer:ident,
        $name:ident: {
            $($subtree:tt)*
        }
        $($rest:tt)*
    ) => {{
        $writer.new_node(stringify!($name)).await?;
        write_v7400_binary! { @__node, $writer, $($subtree)* }
        $writer.close_node().await?;
        write_v7400_binary! { @__node, $writer, $($rest)* }
    }};
    (@__node, $writer:ident,
        $name:ident: [$($attr:expr),* $(,)?] {
            $($subtree:tt)*
        }
        $($rest:tt)*
    ) => {{
        let mut _attrs = $writer.new_node(stringify!($name)).await?;
        $({
            let attr = $attr;
            write_v7400_binary!(@__attr, _attrs, attr.into());
        })*
        write_v7400_binary! { @__node, $writer, $($subtree)* }
        $writer.close_node().await?;
        write_v7400_binary! { @__node, $writer, $($rest)* }
    }};
    (@__node, $writer:ident,
        $name:ident: ($attrs:expr) {
            $($subtree:tt)*
        }
        $($rest:tt)*
    ) => {{
        let mut _attrs = $writer.new_node(stringify!($name)).await?;
        for attr in $attrs.into_iter() {
            write_v7400_binary!(@__attr, _attrs, attr)
        };
        write_v7400_binary! { @__node, $writer, $($subtree)* }
        $writer.close_node().await?;
        write_v7400_binary! { @__node, $writer, $($rest)* }
    }};

    (@__attr, $attrs:ident, $attr:expr) => {{
        use fbxcel_low::v7400::AttributeValue::*;
        match $attr {
            Bool(v) => $attrs.append_bool(v).await?,
            I16(v) => $attrs.append_i16(v).await?,
            I32(v) => $attrs.append_i32(v).await?,
            I64(v) => $attrs.append_i64(v).await?,
            F32(v) => $attrs.append_f32(v).await?,
            F64(v) => $attrs.append_f64(v).await?,
            ArrBool(v) => $attrs.append_arr_bool_from_iter(None, v).await?,
            ArrI32(v) => $attrs.append_arr_i32_from_iter(None, v).await?,
            ArrI64(v) => $attrs.append_arr_i64_from_iter(None, v).await?,
            ArrF32(v) => $attrs.append_arr_f32_from_iter(None, v).await?,
            ArrF64(v) => $attrs.append_arr_f64_from_iter(None, v).await?,
            Binary(v) => $attrs.append_binary_direct(&v).await?,
            String(v) => $attrs.append_string_direct(&v).await?,
        }
    }};
}

#[cfg(test)]
mod tests {
    use crate::v7400::binary::{Result, Writer};
    use fbxcel_low::FbxVersion;
    use futures_util::io::Cursor;

    #[async_std::test]
    async fn empty_writer() -> Result<()> {
        let mut writer = Writer::new(Cursor::new(Vec::new()), FbxVersion::V7_4).await?;
        write_v7400_binary!(writer = writer, tree = {},)?;
        let _buf = writer.finalize_and_flush(&Default::default()).await?;

        Ok(())
    }

    #[async_std::test]
    async fn empty_node() -> Result<()> {
        let mut writer = Writer::new(Cursor::new(Vec::new()), FbxVersion::V7_4).await?;
        write_v7400_binary!(
            writer=writer,
            tree={
                Hello: {}
                World: {},
            },
        )?;
        let _buf = writer.finalize_and_flush(&Default::default()).await?;

        Ok(())
    }

    #[async_std::test]
    async fn nested_node() -> Result<()> {
        let mut writer = Writer::new(Cursor::new(Vec::new()), FbxVersion::V7_4).await?;
        write_v7400_binary!(
            writer=writer,
            tree={
                Hello: {
                    Hello1: {},
                    Hello2: {}
                }
                World: {
                    World1: {
                        World1_1: {}
                        World1_2: {}
                    }
                    World2: {},
                },
            },
        )?;
        let _buf = writer.finalize_and_flush(&Default::default()).await?;

        Ok(())
    }

    #[async_std::test]
    async fn nested_node_with_attrs() -> Result<()> {
        let mut writer = Writer::new(Cursor::new(Vec::new()), FbxVersion::V7_4).await?;
        write_v7400_binary!(
            writer=writer,
            tree={
                Hello: {
                    Hello1: (vec!["string".into()]) {},
                    Hello2: [1.234f32, 42i64] {}
                }
                World: {
                    World1: {
                        World1_1: (vec!["Hello".into(), 42i32.into()]) {}
                        World1_2: [] {}
                    }
                    World2: {},
                },
            },
        )?;
        let _buf = writer.finalize_and_flush(&Default::default()).await?;

        Ok(())
    }
}
