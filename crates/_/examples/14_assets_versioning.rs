use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::{file::FileAssetFetch, rewrite::RewriteAssetFetch},
    protocol::text::TextAssetProtocol,
};
use std::error::Error;

// Current assets version.
const VERSION: &str = "2";

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        // Rewrite asset fetch allows to rewrite input asset paths to some other
        // before inner fetch tries to load it.
        .with_fetch(RewriteAssetFetch::new(
            FileAssetFetch::default().with_root("resources"),
            |path| {
                // Get version from requested asset meta items or use current version.
                let version = path
                    .meta_items()
                    .find(|(key, _)| *key == "v")
                    .map(|(_, value)| value)
                    .unwrap_or(VERSION);
                // Build new asset path that includes version.
                // Example: `protocol://dir/asset.v1.ext`
                Ok(AssetPath::from_parts(
                    path.protocol(),
                    &format!(
                        "{}.v{}{}",
                        path.path_without_extension(),
                        version,
                        path.path_dot_extension().unwrap_or_default()
                    ),
                    path.meta(),
                ))
            },
        ));

    // Gets `text://versioned.v1.txt`.
    let v1 = database.ensure("text://versioned.txt?v=1")?;
    println!("Version 1: {}", v1.access::<&String>(&database));

    // Gets `text://versioned.v2.txt`.
    let v2 = database.ensure("text://versioned.txt")?;
    println!("Version 2: {}", v2.access::<&String>(&database));

    Ok(())
}
