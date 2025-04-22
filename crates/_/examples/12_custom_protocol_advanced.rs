use anput::world::World;
use keket::{
    database::{
        AssetDatabase,
        handle::{AssetDependency, AssetHandle},
        path::AssetPathStatic,
    },
    fetch::{AssetAwaitsResolution, AssetBytesAreReadyToProcess, file::FileAssetFetch},
    protocol::AssetProtocol,
};
use serde::Deserialize;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        // Register custom asset protocol.
        .with_protocol(CustomAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"))
        .with_event(|event| {
            println!("Asset closure event: {:#?}", event);
            Ok(())
        });

    let handle = database.ensure("custom://part1.json")?;

    while database.is_busy() {
        database.maintain()?;
    }

    let contents = handle.access::<&CustomAsset>(&database).contents(&database);
    println!("Custom chain contents: {:?}", contents);

    Ok(())
}

// Custom asset type.
#[derive(Debug, Default, Deserialize)]
struct CustomAsset {
    content: String,
    #[serde(default)]
    next: Option<AssetPathStatic>,
}

impl CustomAsset {
    fn contents(&self, database: &AssetDatabase) -> String {
        let mut result = String::new();
        let mut current = Some(self);
        while let Some(asset) = current {
            result.push_str(asset.content.as_str());
            current = current
                .as_ref()
                .and_then(|asset| asset.next.as_ref())
                .and_then(|path| path.find(database))
                .and_then(|handle| handle.access_checked::<&Self>(database));
            if current.is_some() {
                result.push(' ');
            }
        }
        result
    }
}

/* ANCHOR: custom_asset_protocol */
struct CustomAssetProtocol;

impl AssetProtocol for CustomAssetProtocol {
    fn name(&self) -> &str {
        "custom"
    }

    fn process_asset_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        // Always remember that in order for asset to be considered done with processing
        // bytes, it has to remove AssetBytesAreReadyToProcess component from that asset.
        // We are doing that by taking its bytes content first and removing it after.
        let bytes = {
            let mut bytes =
                storage.component_mut::<true, AssetBytesAreReadyToProcess>(handle.entity())?;
            std::mem::take(&mut bytes.0)
        };
        storage.remove::<(AssetBytesAreReadyToProcess,)>(handle.entity())?;

        // Once we have asset bytes, we decode them into asset data.
        let asset = serde_json::from_slice::<CustomAsset>(&bytes)?;

        // We also have to extract dependencies if it has some.
        if let Some(path) = asset.next.clone() {
            // For every dependency we get, we need to spawn an asset entity with
            // AssetAwaitsResolution component to tell asset database to start loading it.
            let entity = storage.spawn((path, AssetAwaitsResolution))?;

            // We should also relate processed asset with its dependency asset.
            // Dependency relations are useful for traversing asset graphs.
            storage.relate::<true, _>(AssetDependency, handle.entity(), entity)?;
        }

        storage.insert(handle.entity(), (asset,))?;
        Ok(())
    }
}
/* ANCHOR_END: custom_asset_protocol */
