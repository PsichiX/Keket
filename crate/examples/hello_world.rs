use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::{bytes::BytesAssetProtocol, serde::SerdeAssetProtocol, text::TextAssetProtocol},
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
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(SerdeAssetProtocol::new("json", |bytes| {
            Ok(serde_json::from_slice::<Value>(&bytes)?)
        }))
        .with_protocol(SerdeAssetProtocol::new("person", |bytes| {
            Ok(serde_json::from_slice::<Person>(&bytes)?)
        }))
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    let lorem = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    let json = database.ensure("json://person.json")?;
    println!("JSON: {:#}", json.access::<&Value>(&database));

    let person = database.ensure("person://person.json")?;
    println!("Person: {:#?}", person.access::<&Person>(&database));

    let trash = database.ensure("bytes://trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

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
