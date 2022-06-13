//! Writer and parser test.
#![cfg(feature = "writer")]
use fbxcel::{
    low::{v7400::AttributeValue, FbxVersion},
    pull_parser::{
        any::{from_seekable_reader, AnyParser},
        v7400::attribute::loaders::DirectLoader,
    },
    writer::v7400::binary::{FbxFooter, Writer},
    writer::write_v7400_binary,
};
use futures_util::io::Cursor;
use std::{
    iter,
    sync::{Arc, Mutex},
};

use self::v7400::writer::{
    expect_fbx_end, expect_node_end, expect_node_start, CUSTOM_UNKNOWN1, MAGIC, UNKNOWN3,
};

mod v7400;

/// Compares expected binary and binary generated with events.
#[async_std::test]
async fn empty_write_v7400() -> Result<(), Box<dyn std::error::Error>> {
    let mut dest = Vec::new();
    let cursor = Cursor::new(&mut dest);
    let writer = Writer::new(cursor, FbxVersion::V7_4).await?;
    let footer = FbxFooter {
        unknown1: Some(&CUSTOM_UNKNOWN1),
        padding_len: Default::default(),
        unknown2: None,
        unknown3: None,
    };
    writer.finalize_and_flush(&footer).await?;

    let expected = {
        let raw_ver = 7400u32;
        let mut vec = Vec::new();
        // Header.
        {
            // Magic.
            vec.extend(MAGIC);
            // Version.
            vec.extend(&raw_ver.to_le_bytes());
        }
        // No nodes.
        {
            // End of implicit root.
            {
                vec.extend(iter::repeat(0).take(4 * 3 + 1));
            }
        }
        // Footer.
        {
            // Footer: unknown1.
            vec.extend(&CUSTOM_UNKNOWN1);
            // Footer: padding.
            {
                let len = vec.len().wrapping_neg() % 16;
                assert_eq!((vec.len() + len) % 16, 0);
                vec.extend(iter::repeat(0).take(len));
            }
            // Footer: unknown2.
            vec.extend(&[0; 4]);
            // Footer: FBX version.
            vec.extend(&raw_ver.to_le_bytes());
            // Footer: 120 zeroes.
            vec.extend(iter::repeat(0).take(120));
            // Footer: unknown3.
            vec.extend(&UNKNOWN3);
        }
        vec
    };

    assert_eq!(dest.len() % 16, 0);
    assert_eq!(dest, expected);

    let mut parser = match from_seekable_reader(Cursor::new(dest)).await? {
        AnyParser::V7400(parser) => parser,
        _ => panic!("Generated data should be parsable with v7400 parser"),
    };
    let warnings = Arc::new(Mutex::new(Vec::new()));
    parser.set_warning_handler({
        let warnings = warnings.clone();
        move |warning, _pos| {
            warnings.lock().unwrap().push(warning);
            Ok(())
        }
    });
    assert_eq!(parser.fbx_version(), FbxVersion::V7_4);

    {
        let footer_res = expect_fbx_end(&mut parser).await?;
        let footer = footer_res?;
        assert_eq!(footer.unknown1, CUSTOM_UNKNOWN1);
        assert_eq!(footer.unknown2, [0u8; 4]);
        assert_eq!(footer.unknown3, UNKNOWN3);
    }

    assert_eq!(warnings.lock().unwrap().len(), 0);

    Ok(())
}

/// Compares expected binary and binary generated with events.
#[async_std::test]
async fn tree_write_v7500() -> Result<(), Box<dyn std::error::Error>> {
    let mut dest = Vec::new();
    let cursor = Cursor::new(&mut dest);
    let mut writer = Writer::new(cursor, FbxVersion::V7_5).await?;
    write_v7400_binary!(
        writer=writer,
        tree={
            Node0: {
                Node0_0: {},
                Node0_1: {},
            },
            Node1: [true] {
                Node1_0: (vec![42f64.into(), 1.234f64.into()]) {}
                Node1_1: [&[1u8, 2, 4, 8, 16][..], "Hello, world"] {}
            },
        },
    )?;
    writer.finalize_and_flush(&Default::default()).await?;

    let mut parser = match from_seekable_reader(Cursor::new(dest)).await? {
        AnyParser::V7400(parser) => parser,
        _ => panic!("Generated data should be parsable with v7400 parser"),
    };
    let warnings = Arc::new(Mutex::new(Vec::new()));
    parser.set_warning_handler({
        let warnings = warnings.clone();
        move |warning, _pos| {
            warnings.lock().unwrap().push(warning);
            Ok(())
        }
    });
    assert_eq!(parser.fbx_version(), FbxVersion::V7_5);

    {
        let attrs = expect_node_start(&mut parser, "Node0").await?;
        assert_eq!(attrs.total_count(), 0);
    }
    {
        let attrs = expect_node_start(&mut parser, "Node0_0").await?;
        assert_eq!(attrs.total_count(), 0);
    }
    expect_node_end(&mut parser).await?;
    {
        let attrs = expect_node_start(&mut parser, "Node0_1").await?;
        assert_eq!(attrs.total_count(), 0);
    }
    expect_node_end(&mut parser).await?;
    expect_node_end(&mut parser).await?;
    {
        let attrs = expect_node_start(&mut parser, "Node1").await?;
        assert_eq!(attrs.total_count(), 1);
    }
    {
        let attrs = expect_node_start(&mut parser, "Node1_0").await?;
        assert_eq!(attrs.total_count(), 2);
    }
    expect_node_end(&mut parser).await?;
    {
        let attrs = expect_node_start(&mut parser, "Node1_1").await?;
        assert_eq!(attrs.total_count(), 2);
    }
    expect_node_end(&mut parser).await?;
    expect_node_end(&mut parser).await?;

    {
        let footer_res = expect_fbx_end(&mut parser).await?;
        assert!(footer_res.is_ok());
    }

    assert_eq!(warnings.lock().unwrap().len(), 0);

    Ok(())
}

#[async_std::test]
async fn macro_v7400_idempotence() -> Result<(), Box<dyn std::error::Error>> {
    let version = FbxVersion::V7_4;
    let mut writer = Writer::new(Cursor::new(Vec::new()), version).await?;

    write_v7400_binary!(
        writer=writer,
        tree={
            Node0: {
                Node0_0: {},
                Node0_1: {},
            },
            Node1: [true] {
                Node1_0: (vec![42i32.into(), 1.234f64.into()]) {}
                Node1_1: [&[1u8, 2, 4, 8, 16][..], "Hello, world"] {}
            },
        },
    )?;
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
            warnings.lock().unwrap().push(warning);
            Ok(())
        }
    });
    assert_eq!(parser.fbx_version(), version);

    {
        let attrs = expect_node_start(&mut parser, "Node0").await?;
        assert_eq!(attrs.total_count(), 0);
    }
    {
        let attrs = expect_node_start(&mut parser, "Node0_0").await?;
        assert_eq!(attrs.total_count(), 0);
    }
    expect_node_end(&mut parser).await?;
    {
        let attrs = expect_node_start(&mut parser, "Node0_1").await?;
        assert_eq!(attrs.total_count(), 0);
    }
    expect_node_end(&mut parser).await?;
    expect_node_end(&mut parser).await?;
    {
        let mut attrs = expect_node_start(&mut parser, "Node1").await?;
        assert_eq!(
            attrs.load_next(DirectLoader).await?,
            Some(AttributeValue::from(true))
        );
        assert_eq!(attrs.total_count(), 1);
    }
    {
        let mut attrs = expect_node_start(&mut parser, "Node1_0").await?;
        assert_eq!(
            attrs.load_next(DirectLoader).await?,
            Some(AttributeValue::from(42i32))
        );
        assert!(attrs
            .load_next(DirectLoader)
            .await?
            .map_or(false, |attr| attr.strict_eq(&1.234f64.into())));
        assert_eq!(attrs.total_count(), 2);
    }
    expect_node_end(&mut parser).await?;
    {
        let mut attrs = expect_node_start(&mut parser, "Node1_1").await?;
        assert_eq!(
            attrs.load_next(DirectLoader).await?,
            Some(AttributeValue::from(vec![1u8, 2, 4, 8, 16]))
        );
        assert_eq!(
            attrs.load_next(DirectLoader).await?,
            Some(AttributeValue::from("Hello, world"))
        );
        assert_eq!(attrs.total_count(), 2);
    }
    expect_node_end(&mut parser).await?;
    expect_node_end(&mut parser).await?;

    {
        let footer_res = expect_fbx_end(&mut parser).await?;
        assert!(footer_res.is_ok());
    }

    assert_eq!(warnings.lock().unwrap().len(), 0);

    Ok(())
}
