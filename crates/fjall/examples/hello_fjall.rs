use keket::{
    database::AssetDatabase,
    fetch::container::ContainerAssetFetch,
    protocol::{bundle::BundleAssetProtocol, bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use keket_fjall::{third_party::fjall::Config, FjallContainerPartialFetch};
use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes| {
            Ok((serde_json::from_slice::<Value>(&bytes)?,))
        }))
        .with_fetch(ContainerAssetFetch::new(FjallContainerPartialFetch::new(
            Config::new("./resources/database/").open()?,
            "assets",
        )));

    let lorem = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    let json = database.ensure("json://person.json")?;
    println!("JSON: {:#}", json.access::<&Value>(&database));

    let trash = database.ensure("bytes://trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    Ok(())
}
