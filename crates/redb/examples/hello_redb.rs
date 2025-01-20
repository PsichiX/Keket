use keket::{
    database::AssetDatabase,
    fetch::container::ContainerAssetFetch,
    protocol::{bundle::BundleAssetProtocol, bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use keket_redb::{third_party::redb::Database, RedbContainerPartialFetch};
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
        .with_fetch(ContainerAssetFetch::new(RedbContainerPartialFetch::new(
            Database::create("./resources/database.redb")?,
            "assets",
        )));

    let lorem = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    let json = database.ensure("json://person.json")?;
    println!("JSON: {:#}", json.access::<&Value>(&database));

    let trash = database.ensure("bytes://trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
    /* ANCHOR_END: main */

    Ok(())
}
