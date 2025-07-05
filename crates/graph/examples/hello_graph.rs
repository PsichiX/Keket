use keket::{
    database::{AssetDatabase, path::AssetPathStatic},
    fetch::file::FileAssetFetch,
    protocol::bundle::BundleAssetProtocol,
};
use keket_graph::{
    node::AssetNode,
    protocol::{AssetTree, AssetTreeProcessor},
};
use keket_graph_derive::AssetTree;
use serde::{Deserialize, Serialize};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(BundleAssetProtocol::new(
            "custom",
            // AssetTreeProcessor handles deserialization of the custom asset
            // as single component. That component will report its dependencies
            // to load.
            AssetTreeProcessor::<CustomAsset>::new(|bytes| {
                Ok(serde_json::from_slice::<CustomAsset>(&bytes)?)
            }),
        ))
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    // Create asset node and ensure it is being loaded.
    let asset = AssetNode::<CustomAsset>::new("custom://part1.json");
    asset.ensure(&mut database)?;

    // Wait until the database is not busy, which means all assets are loaded.
    while database.is_busy() {
        database.maintain()?;
    }

    // Resolve the asset and read its contents.
    // This will also resolve all dependencies of the asset.
    let contents = asset
        .resolve(&database)?
        .read_unchecked()
        .contents(&database);
    println!("Custom chain contents: {contents:?}");
    // /* ANCHOR_END: main */
    Ok(())
}

/* ANCHOR: custom_asset */
// This example demonstrates how to create a custom asset type that can
// have dependencies on other assets. The `CustomAsset` struct is defined
// with a `content` field and an optional `next` field that points to another
// `CustomAsset`.
#[derive(Debug, Default, Serialize, Deserialize, AssetTree)]
struct CustomAsset {
    content: String,
    // The `next` field is annotated with `#[asset_deps]` to indicate that it is
    // an asset dependency that has to be loaded along with this asset.
    #[serde(default)]
    #[asset_deps]
    next: Option<AssetNode<CustomAsset>>,
}

impl CustomAsset {
    fn contents(&self, database: &AssetDatabase) -> String {
        let mut result = self.content.as_str().to_owned();
        if let Some(next) = self.next.as_ref() {
            result.push(' ');
            if let Ok(resolved) = next.resolve(database) {
                result.push_str(&resolved.read_unchecked().contents(database));
            }
        }
        result
    }
}
/* ANCHOR_END: custom_asset */
