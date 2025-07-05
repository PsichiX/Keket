use keket::{
    database::{AssetDatabase, path::AssetPathStatic},
    fetch::file::FileAssetFetch,
    protocol::bundle::{
        BundleAssetProtocol, BundleWithDependencies, BundleWithDependenciesProcessor,
    },
};
use serde::Deserialize;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        // Register custom asset processor.
        .with_protocol(BundleAssetProtocol::new("custom", CustomAssetProcessor))
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    // Ensure custom asset existence.
    let handle = database.ensure("custom://part1.json")?;

    // Make database process loaded dependencies.
    while database.is_busy() {
        database.maintain()?;
    }

    let contents = handle.access::<&CustomAsset>(&database).contents(&database);
    println!("Custom chain contents: {contents:?}");

    Ok(())
}

/* ANCHOR: custom_asset */
// Custom asset type.
#[derive(Debug, Default, Deserialize)]
struct CustomAsset {
    // Main content.
    content: String,
    // Optional sibling asset (content continuation).
    #[serde(default)]
    next: Option<AssetPathStatic>,
}
/* ANCHOR_END: custom_asset */

impl CustomAsset {
    fn contents(&self, database: &AssetDatabase) -> String {
        // Read this and it's siblings content to output.
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
struct CustomAssetProcessor;

impl BundleWithDependenciesProcessor for CustomAssetProcessor {
    // Bundle of asset components this asset processor produces from processed asset.
    type Bundle = (CustomAsset,);

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        let asset = serde_json::from_slice::<CustomAsset>(&bytes)?;
        let dependency = asset.next.clone();
        // Return bundled components and optional dependency which will be additionally loaded.
        Ok(BundleWithDependencies::new((asset,)).maybe_dependency(dependency))
    }
}
/* ANCHOR_END: custom_asset_protocol */
