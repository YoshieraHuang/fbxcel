use std::path::PathBuf;

use async_std::fs::File;
use fbxcel::{
    low::v7400::AttributeValue,
    pull_parser::{
        any::{from_seekable_reader, AnyParser},
        AsyncPositionRead,
    },
};
use futures_util::{io::BufReader, AsyncBufRead};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    match std::env::args_os().nth(1) {
        None => {
            eprintln!("Usage: dump-pull-parser-events <FBX_FILE>");
        }
        Some(v) => {
            let path = PathBuf::from(v);
            let file = File::open(path).await?;
            let reader = BufReader::new(file);

            match from_seekable_reader(reader).await? {
                AnyParser::V7400(mut parser) => {
                    let version = parser.fbx_version();
                    println!("FBX version: {}.{}", version.major(), version.minor());
                    parser.set_warning_handler(|w, pos| {
                        eprintln!("WARNING: {} (pos={:?})", w, pos);
                        Ok(())
                    });
                    dump_fbx_7400(parser).await?;
                }
                parser => panic!(
                    "Unsupported by this example: fbx_version={:?}",
                    parser.fbx_version()
                ),
            }
        }
    };

    Ok(())
}

fn indent(depth: usize) {
    print!("{:depth$}", "", depth = depth * 4);
}

async fn dump_fbx_7400<R: AsyncPositionRead + AsyncBufRead + Unpin + Send>(
    mut parser: fbxcel_pull_parser::v7400::Parser<R>,
) -> fbxcel_pull_parser::Result<()> {
    let mut depth = 0;

    /// Dump format of node attributes.
    enum AttrsDumpFormat {
        /// Type only.
        Type,
        /// Value for primitive types, length for array, binary, and string.
        Length,
        /// Values for all types.
        ///
        /// Not recommended because the output might be quite large.
        Full,
    }

    let attrs_dump_format = match std::env::var("DUMP_ATTRIBUTES").as_ref().map(AsRef::as_ref) {
        Ok("length") => AttrsDumpFormat::Length,
        Ok("full") => AttrsDumpFormat::Full,
        _ => AttrsDumpFormat::Type,
    };

    loop {
        use fbxcel_pull_parser::v7400::*;

        match parser.next_event().await? {
            Event::StartNode(start) => {
                indent(depth);
                println!("Node start: {:?}", start.name());
                depth += 1;

                let attrs = start.attributes();
                match attrs_dump_format {
                    AttrsDumpFormat::Type => dump_v7400_attributes_type(depth, attrs).await?,
                    AttrsDumpFormat::Length => dump_v7400_attributes_length(depth, attrs).await?,
                    AttrsDumpFormat::Full => dump_v7400_attributes_full(depth, attrs).await?,
                }
            }
            Event::EndNode => {
                depth -= 1;
                indent(depth);
                println!("Node end");
            }
            Event::EndFbx(footer_res) => {
                println!("FBX end");
                match footer_res {
                    Ok(footer) => println!("footer: {:#?}", footer),
                    Err(e) => println!("footer has an error: {:?}", e),
                }
                break;
            }
        }
    }

    Ok(())
}

async fn dump_v7400_attributes_length<R>(
    depth: usize,
    mut attrs: fbxcel_pull_parser::v7400::Attributes<'_, R>,
) -> fbxcel_pull_parser::Result<()>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
{
    use fbxcel_pull_parser::v7400::attribute::loaders::DirectLoader;

    while let Some(attr) = attrs.load_next(DirectLoader).await? {
        let type_ = attr.attribute_type();
        indent(depth);
        match attr {
            AttributeValue::Bool(_) => println!("Attribute: {:?}", attr),
            AttributeValue::I16(_) => println!("Attribute: {:?}", attr),
            AttributeValue::I32(_) => println!("Attribute: {:?}", attr),
            AttributeValue::I64(_) => println!("Attribute: {:?}", attr),
            AttributeValue::F32(_) => println!("Attribute: {:?}", attr),
            AttributeValue::F64(_) => println!("Attribute: {:?}", attr),
            AttributeValue::ArrBool(v) => println!("Attribute: type={:?}, len={}", type_, v.len()),
            AttributeValue::ArrI32(v) => println!("Attribute: type={:?}, len={}", type_, v.len()),
            AttributeValue::ArrI64(v) => println!("Attribute: type={:?}, len={}", type_, v.len()),
            AttributeValue::ArrF32(v) => println!("Attribute: type={:?}, len={}", type_, v.len()),
            AttributeValue::ArrF64(v) => println!("Attribute: type={:?}, len={}", type_, v.len()),
            AttributeValue::Binary(v) => println!("Attribute: type={:?}, len={}", type_, v.len()),
            AttributeValue::String(v) => println!("Attribute: type={:?}, len={}", type_, v.len()),
        }
    }

    Ok(())
}

async fn dump_v7400_attributes_type<R>(
    depth: usize,
    mut attrs: fbxcel_pull_parser::v7400::Attributes<'_, R>,
) -> fbxcel_pull_parser::Result<()>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
{
    use fbxcel_pull_parser::v7400::attribute::loaders::TypeLoader;

    while let Some(r#type) = attrs.load_next(TypeLoader).await? {
        indent(depth);
        println!("Attribute: {:?}", r#type);
    }

    Ok(())
}

async fn dump_v7400_attributes_full<R>(
    depth: usize,
    mut attrs: fbxcel_pull_parser::v7400::Attributes<'_, R>,
) -> fbxcel_pull_parser::Result<()>
where
    R: AsyncPositionRead + AsyncBufRead + Unpin + Send,
{
    use fbxcel_pull_parser::v7400::attribute::loaders::DirectLoader;

    while let Some(attr) = attrs.load_next(DirectLoader).await? {
        let attribute_type = attr.attribute_type();
        indent(depth);
        match attr {
            AttributeValue::Bool(_) => println!("Attribute: {:?}", attr),
            AttributeValue::I16(_) => println!("Attribute: {:?}", attr),
            AttributeValue::I32(_) => println!("Attribute: {:?}", attr),
            AttributeValue::I64(_) => println!("Attribute: {:?}", attr),
            AttributeValue::F32(_) => println!("Attribute: {:?}", attr),
            AttributeValue::F64(_) => println!("Attribute: {:?}", attr),
            AttributeValue::ArrBool(v) => println!(
                "Attribute: type={:?}, len={}, value={:?}",
                attribute_type,
                v.len(),
                v
            ),
            AttributeValue::ArrI32(v) => println!(
                "Attribute: type={:?}, len={}, value={:?}",
                attribute_type,
                v.len(),
                v
            ),
            AttributeValue::ArrI64(v) => println!(
                "Attribute: type={:?}, len={}, value={:?}",
                attribute_type,
                v.len(),
                v
            ),
            AttributeValue::ArrF32(v) => println!(
                "Attribute: type={:?}, len={}, value={:?}",
                attribute_type,
                v.len(),
                v
            ),
            AttributeValue::ArrF64(v) => println!(
                "Attribute: type={:?}, len={}, value={:?}",
                attribute_type,
                v.len(),
                v
            ),
            AttributeValue::Binary(v) => println!(
                "Attribute: type={:?}, len={}, value={:?}",
                attribute_type,
                v.len(),
                v
            ),
            AttributeValue::String(v) => println!(
                "Attribute: type={:?}, len={}, value={:?}",
                attribute_type,
                v.len(),
                v
            ),
        }
    }

    Ok(())
}
