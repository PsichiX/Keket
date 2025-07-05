use keket::{
    database::{AssetDatabase, reference::AssetRef},
    fetch::file::FileAssetFetch,
    protocol::bundle::{
        BundleAssetProtocol, BundleWithDependencies, BundleWithDependenciesProcessor,
    },
};
use serde::Deserialize;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(BundleAssetProtocol::new("custom", CustomAssetProcessor))
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    let handle = database.ensure("custom://part1.json")?;

    while database.is_busy() {
        database.maintain()?;
    }

    let contents = handle.access::<&CustomAsset>(&database).contents(&database);
    println!("Custom chain contents: {contents:?}");
    /* ANCHOR_END: main */

    Ok(())
}

/* ANCHOR: custom_asset */
#[derive(Debug, Default, Deserialize)]
struct CustomAsset {
    content: String,
    // Asset references are used to store path and cached handle. They serialize as asset paths.
    // They can be used where we need to have an ability to reference asset by path, and ask
    // for handle once instead of everytime as with asset paths.
    #[serde(default)]
    next: Option<AssetRef>,
}

impl CustomAsset {
    fn contents(&self, database: &AssetDatabase) -> String {
        let mut result = self.content.as_str().to_owned();
        if let Some(next) = self.next.as_ref() {
            result.push(' ');
            if let Ok(resolved) = next.resolve(database) {
                result.push_str(&resolved.access::<&Self>().contents(database));
            }
        }
        result
    }
}
/* ANCHOR_END: custom_asset */

struct CustomAssetProcessor;

impl BundleWithDependenciesProcessor for CustomAssetProcessor {
    type Bundle = (CustomAsset,);

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        let asset = serde_json::from_slice::<CustomAsset>(&bytes)?;
        let dependency = asset
            .next
            .as_ref()
            .map(|reference| reference.path().clone());
        Ok(BundleWithDependencies::new((asset,)).maybe_dependency(dependency))
    }
}
