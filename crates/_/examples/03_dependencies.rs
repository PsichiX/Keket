use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::{
        bundle::BundleAssetProtocol,
        bytes::BytesAssetProtocol,
        group::{GroupAsset, GroupAssetProtocol},
        text::TextAssetProtocol,
    },
};
use serde_json::Value;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes| {
            Ok((serde_json::from_slice::<Value>(&bytes)?,))
        }))
        .with_protocol(GroupAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    let group = database.ensure("group://group.txt")?;
    println!("Group: {:#?}", group.access::<&GroupAsset>(&database));

    database.maintain()?;

    let lorem = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    let json = database.ensure("json://person.json")?;
    println!("JSON: {:#}", json.access::<&Value>(&database));

    let trash = database.ensure("bytes://trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    for dependency in group.dependencies(&database) {
        println!(
            "Group dependency: {}",
            dependency.access::<&AssetPath>(&database)
        );
    }

    Ok(())
}
