use keket::{
    database::{handle::AssetHandle, path::AssetPath, AssetDatabase},
    fetch::deferred::DeferredAssetFetch,
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use keket_client::{third_party::reqwest::Url, ClientAssetFetch};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        // Client asset fetch to request files from asset server.
        .with_fetch(DeferredAssetFetch::new(ClientAssetFetch::new(
            // IP address of asset server we connect to.
            "127.0.0.1:8080",
        )?));

    // Ensure assets exists or start getting fetched.
    let lorem = database.ensure("text://lorem.txt")?;
    let trash = database.ensure("bytes://trash.bin")?;

    // Wait for pending fetches.
    while database.does_await_deferred_job() {
        println!("Waiting for database to be free");
        println!(
            "Loading:\n- Lorem Ipsum: {}\n- Bytes: {}",
            lorem.awaits_deferred_job(&database),
            trash.awaits_deferred_job(&database)
        );
        database.maintain()?;
    }

    // After all assets bytes are fetched, process them into assets.
    database.maintain()?;

    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    // List all assets from client.
    for (asset_path, url) in database.storage.query::<true, (&AssetPath, &Url)>() {
        println!("Asset: `{}` at url: `{}`", asset_path, url);
    }

    println!("Listening for file changes...");
    loop {
        database.maintain()?;

        // With storage change detection we can ask for asset entities
        // that their paths were updated (hot reloading updates them).
        if let Some(changes) = database.storage.updated() {
            for entity in changes.iter_of::<AssetPath>() {
                println!(
                    "Asset changed: `{}`",
                    AssetHandle::new(entity).access::<&AssetPath>(&database)
                );
            }
        }

        // Simulate systems that detect particular asset type reload.
        for entity in database.storage.added().iter_of::<String>() {
            println!(
                "Text asset changed: `{}`",
                AssetHandle::new(entity).access::<&String>(&database)
            );
        }
        for entity in database.storage.added().iter_of::<Vec<u8>>() {
            println!(
                "Bytes asset changed: `{:?}`",
                AssetHandle::new(entity).access::<&Vec<u8>>(&database)
            );
        }
    }
}
