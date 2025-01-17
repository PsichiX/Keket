use keket::{
    database::{path::AssetPath, AssetDatabase},
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

    Ok(())
}
