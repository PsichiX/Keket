use keket::{
    database::{AssetDatabase, path::AssetPath},
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
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes: Vec<u8>| {
            Ok((serde_json::from_slice::<Value>(&bytes)?,).into())
        }))
        // Group asset protocol stores paths to other assets that gets scheduled
        // for loading when group loads.
        .with_protocol(GroupAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    let group = database.ensure("group://group.txt")?;
    println!("Group: {:#?}", group.access::<&GroupAsset>(&database));

    while database.is_busy() {
        database.maintain()?;
    }

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
    /* ANCHOR_END: main */

    Ok(())
}
