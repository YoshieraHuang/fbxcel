//! Tests for writer, tree, and parser.
#![cfg(all(feature = "tree", feature = "writer"))]

use std::sync::{Arc, Mutex};

use fbxcel::{
    low::FbxVersion,
    pull_parser::any::{from_seekable_reader, AnyParser},
    tree::tree_v7400,
    tree::v7400::Loader as TreeLoader,
    writer::v7400::binary::Writer,
};
use futures_lite::io::Cursor;

/// Construct tree, export it to binary, parse it and construct tree, and
/// compare them.
#[async_std::test]
async fn tree_write_parse_idempotence_v7500() -> Result<(), Box<dyn std::error::Error>> {
    // Construct tree.
    let tree1 = tree_v7400! {
        Node0: {
            Node0_0: {},
            Node0_1: {},
        },
        Node1: [true] {
            Node1_0: (vec![42i32.into(), 1.234f64.into()]) {}
            Node1_1: [&[1u8, 2, 4, 8, 16][..], "Hello, world"] {}
        },
    };

    let mut writer = Writer::new(Cursor::new(Vec::new()), FbxVersion::V7_5).await?;
    writer.write_tree(&tree1).await?;
    let bin = writer
        .finalize_and_flush(&Default::default())
        .await?
        .into_inner();

    let mut parser = match from_seekable_reader(Cursor::new(bin)).await? {
        AnyParser::V7400(parser) => parser,
        _ => panic!("Generated data should be parsable with v7400 parser"),
    };
    let warnings = Arc::new(Mutex::new(Vec::new()));
    parser.set_warning_handler({
        let warnings = warnings.clone();
        move |warning, _pos| {
            let mut warnings = warnings.lock().unwrap();
            warnings.push(warning);
            Ok(())
        }
    });
    assert_eq!(parser.fbx_version(), FbxVersion::V7_5);

    let (tree2, footer_res) = TreeLoader::new().load(&mut parser).await?;

    assert_eq!(warnings.lock().unwrap().len(), 0);
    assert!(footer_res.is_ok());

    assert!(tree1.strict_eq(&tree2));

    Ok(())
}
