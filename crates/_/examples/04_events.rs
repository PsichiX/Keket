use keket::{
    database::{events::AssetEventBindings, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::{
        bundle::BundleAssetProtocol, bytes::BytesAssetProtocol, group::GroupAssetProtocol,
        text::TextAssetProtocol,
    },
};
use serde_json::Value;
use std::{error::Error, sync::mpsc::channel};

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_protocol(BundleAssetProtocol::new("json", |bytes: Vec<u8>| {
            let asset = serde_json::from_slice::<Value>(&bytes)?;
            Ok((asset,).into())
        }))
        .with_protocol(GroupAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    // We can bind closures to asset event bindings for any asset progression tracking.
    database.events.bind(|event| {
        println!("Asset closure event: {:?}", event);
        Ok(())
    });

    // Create channel for asset events communication.
    let (tx, rx) = channel();

    // Start loading asset and its dependencies.
    let group = database.ensure("group://group.txt")?;
    // We can also bind sender to asset event bindings.
    group.ensure::<AssetEventBindings>(&mut database)?.bind(tx);

    while database.is_busy() {
        database.maintain()?;
    }

    // Read sent events from receiver.
    while let Ok(event) = rx.try_recv() {
        println!("Group channel event: {:?}", event);
    }

    Ok(())
}
