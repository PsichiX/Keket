use keket::{
    database::{path::AssetPath, reference::AssetRef, AssetDatabase},
    fetch::{file::FileAssetFetch, hotreload::HotReloadFileAssetFetch},
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        // Hot reload wrapper watches for changes in file fetch root path.
        .with_fetch(HotReloadFileAssetFetch::new(
            FileAssetFetch::default().with_root("resources"),
        )?);

    // First we fill database with some assets, hot reload only
    // cares about changes in files present in database.
    database.ensure("text://lorem.txt")?;
    database.ensure("bytes://trash.bin")?;

    println!("Watching for file changes...");
    loop {
        database.maintain()?;

        // With storage change detection we can ask for asset entities
        // that their paths were updated (hot reloading updates them).
        if let Some(changes) = database.storage.updated() {
            for entity in changes.iter_of::<AssetPath>() {
                println!(
                    "Asset changed: `{}`",
                    AssetRef::new(entity).access::<&AssetPath>(&database)
                );
            }
        }

        // Simulate systems that detect particular asset type reload.
        for entity in database.storage.added().iter_of::<String>() {
            println!(
                "Text asset changed: `{}`",
                AssetRef::new(entity).access::<&String>(&database)
            );
        }
        for entity in database.storage.added().iter_of::<Vec<u8>>() {
            println!(
                "Bytes asset changed: `{:?}`",
                AssetRef::new(entity).access::<&Vec<u8>>(&database)
            );
        }
    }
}
