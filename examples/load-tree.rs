use std::path::PathBuf;

use async_std::fs::File;
use fbxcel::tree::any::AnyTree;
use futures_lite::io::BufReader;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    match std::env::args_os().nth(1) {
        None => {
            eprintln!("Usage: load-tree <FBX_FILE>");
        }
        Some(v) => {
            let path = PathBuf::from(v);
            let file = File::open(path).await?;
            let reader = BufReader::new(file);

            match AnyTree::from_seekable_reader(reader).await? {
                AnyTree::V7400(fbx_version, tree, footer) => {
                    println!("FBX version = {:#?}", fbx_version);
                    println!("tree = {:#?}", tree);
                    println!("footer = {:#?}", footer);
                }
                _ => panic!("FBX version unsupported by this example"),
            };
        }
    };

    Ok(())
}
