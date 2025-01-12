use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::deferred::DeferredAssetFetch,
    protocol::{bundle::BundleAssetProtocol, bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use keket_http::{third_party::reqwest::Url, HttpAssetFetch};
use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes| {
            Ok((serde_json::from_slice::<Value>(&bytes)?,))
        }))
        // HTTP asset fetch with root URL for assets.
        .with_fetch(DeferredAssetFetch::new(HttpAssetFetch::new(
            "https://raw.githubusercontent.com/PsichiX/Keket/refs/heads/master/resources/",
        )?));

    // ENsure assets exists or start getting fetched.
    let lorem = database.ensure("text://lorem.txt")?;
    let json = database.ensure("json://person.json")?;
    let trash = database.ensure("bytes://trash.bin")?;

    // Wait for pending fetches.
    while database.does_await_deferred_job() {
        println!("Waiting for database to be free");
        println!(
            "Loading:\n- Lorem Ipsum: {}\n- JSON: {}\n- Bytes: {}",
            lorem.awaits_deferred_job(&database),
            json.awaits_deferred_job(&database),
            trash.awaits_deferred_job(&database)
        );
        database.maintain()?;
    }

    // After all assets bytes are fetched, process them into assets.
    database.maintain()?;

    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
    println!("JSON: {:#}", json.access::<&Value>(&database));
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    // List all assets from HTTP.
    for (asset_path, url) in database.storage.query::<true, (&AssetPath, &Url)>() {
        println!("Asset: `{}` at url: `{}`", asset_path, url);
    }

    Ok(())
}
