use anput::bundle::DynamicBundle;
use keket::{
    database::{AssetDatabase, path::AssetPath},
    fetch::{AssetBytesAreReadyToProcess, AssetFetch},
    protocol::text::TextAssetProtocol,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        // Register custom asset fetch.
        .with_fetch(CustomAssetFetch([(
            "lorem.txt",
            include_bytes!("../../../resources/lorem.txt"),
        )]));

    let handle = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", handle.access::<&String>(&database));

    Ok(())
}

struct AssetFromCustomSource;

struct CustomAssetFetch<const N: usize>([(&'static str, &'static [u8]); N]);

impl<const N: usize> AssetFetch for CustomAssetFetch<N> {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        for (name, bytes) in self.0 {
            if path.path() == name {
                // Once asset bytes are found, we need to build and return dynamic bundle.
                let mut bundle = DynamicBundle::default();
                // Make sure to add AssetBytesAreReadyToProcess with fetched bytes.
                // This component tells asset database to process this asset with dedicated protocol.
                bundle
                    .add_component(AssetBytesAreReadyToProcess(bytes.to_vec()))
                    .ok()
                    .unwrap();
                // At this point you can also add any meta data that your fetch engine considers
                // useful for assets made by this fetch engine. It's good practice to have tag
                // component that can be used in queries that care about assets made by fetch engine.
                bundle.add_component(AssetFromCustomSource).ok().unwrap();
                return Ok(bundle);
            }
        }
        Err(format!("Missing asset: `{path}`").into())
    }
}
