use keket::{
    database::{AssetDatabase, path::AssetPath},
    fetch::deferred::DeferredAssetFetch,
    protocol::{bundle::BundleAssetProtocol, bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use keket_http::{HttpAssetFetch, third_party::reqwest::Url};
use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes: Vec<u8>| {
            Ok((serde_json::from_slice::<Value>(&bytes)?,).into())
        }))
        // HTTP asset fetch with root URL for assets.
        .with_fetch(DeferredAssetFetch::new(HttpAssetFetch::new(
            "https://raw.githubusercontent.com/PsichiX/Keket/refs/heads/master/resources/",
        )?));

    // Ensure assets exists or start getting fetched.
    let lorem = database.ensure("text://lorem.txt")?;
    let json = database.ensure("json://person.json")?;
    let trash = database.ensure("bytes://trash.bin")?;

    // Wait while database is busy.
    while database.is_busy() {
        println!("Waiting for database to be free");
        println!(
            "Loading:\n- Lorem Ipsum: {}\n- JSON: {}\n- Bytes: {}",
            lorem.awaits_async_fetch(&database),
            json.awaits_async_fetch(&database),
            trash.awaits_async_fetch(&database)
        );
        database.maintain()?;
    }

    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
    println!("JSON: {:#}", json.access::<&Value>(&database));
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    // List all assets from HTTP.
    for (asset_path, url) in database.storage.query::<true, (&AssetPath, &Url)>() {
        println!("Asset: `{asset_path}` at url: `{url}`");
    }
    /* ANCHOR_END: main */

    Ok(())
}
