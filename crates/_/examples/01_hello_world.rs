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
    let mut database = AssetDatabase::default()
        // Asset protocols tell how to deserialize bytes into assets.
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes| {
            Ok((serde_json::from_slice::<Value>(&bytes)?,))
        }))
        .with_protocol(BundleAssetProtocol::new("person", |bytes| {
            Ok((serde_json::from_slice::<Person>(&bytes)?,))
        }))
        // Asset fetch tells how to get bytes from specific source.
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    // Ensure method either gives existing asset handle or loads if not existent.
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

    Ok(())
}
