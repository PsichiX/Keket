use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::{bundle::BundleAssetProtocol, bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use serde::Deserialize;
use serde_json::Value;
use std::{error::Error, fs::Metadata, path::PathBuf};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Person {
    age: u8,
    home: PersonHome,
    friends: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PersonHome {
    country: String,
    address: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        // Asset protocols tell how to deserialize bytes into assets.
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        // Bundle protocol allows for easly making a protocol that takes
        // bytes and returns bundle with optional dependencies.
        .with_protocol(BundleAssetProtocol::new("json", |bytes: Vec<u8>| {
            let asset = serde_json::from_slice::<Value>(&bytes)?;
            Ok((asset,).into())
        }))
        .with_protocol(BundleAssetProtocol::new("person", |bytes: Vec<u8>| {
            let asset = serde_json::from_slice::<Person>(&bytes)?;
            Ok((asset,).into())
        }))
        // Asset fetch tells how to get bytes from specific source.
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    /* ANCHOR: handle */
    // Ensure method either gives existing asset handle or creates new
    // and loads asset if not existing yet in storage.
    let lorem = database.ensure("text://lorem.txt")?;
    // Accessing component(s) of asset entry.
    // Assets can store multiple data associated to them, consider them meta data.
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    let json = database.ensure("json://person.json")?;
    println!("JSON: {:#}", json.access::<&Value>(&database));

    let person = database.ensure("person://person.json")?;
    println!("Person: {:#?}", person.access::<&Person>(&database));

    let trash = database.ensure("bytes://trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
    /* ANCHOR_END: handle */

    /* ANCHOR: fs_query */
    // We can query storage for asset components to process assets, just like with ECS.
    for (asset_path, file_path, metadata) in database
        .storage
        .query::<true, (&AssetPath, &PathBuf, &Metadata)>()
    {
        println!(
            "Asset: `{}` at location: {:?} has metadata: {:#?}",
            asset_path, file_path, metadata
        );
    }
    /* ANCHOR_END: fs_query */
    /* ANCHOR_END: main */

    Ok(())
}
