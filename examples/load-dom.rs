use std::{path::PathBuf};

use async_std::{io::BufReader, fs::File};
use fbxcel_dom::any::AnyDocument;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    match std::env::args_os().nth(1) {
        None => {
            eprintln!("Usage: load-dom <FBX_FILE>");
        }
        Some(v) => {
            let path = PathBuf::from(v);
            let file = File::open(path).await?;
            let reader = BufReader::new(file);

            match AnyDocument::from_seekable_reader(reader).await? {
                AnyDocument::V7400(ver, doc) => {
                    println!("Loaded FBX DOM successfully: FBX version = {:?}", ver);
                    for scene in doc.scenes() {
                        println!("Scene object: object_id={:?}", scene.object_id());
                        let root_id = scene
                            .root_object_id()
                            .expect("Failed to get root object ID");
                        println!("\tRoot object ID: {:?}", root_id);
                    }
                }
                _ => panic!("FBX version unsupported by this example"),
            };
        }
    }
    Ok(())
}
