use keket::{
    database::{
        AssetDatabase,
        handle::AssetHandle,
        path::{AssetPath, AssetPathStatic},
    },
    fetch::file::FileAssetFetch,
    protocol::AssetProtocol,
    third_party::anput::{bundle::DynamicBundle, world::World},
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        // Register custom asset protocol.
        .with_protocol(CustomAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"))
        .with_event(|event| {
            println!("Asset closure event: {event:#?}");
            Ok(())
        });

    // We spawn an asset with configuration meta to be extracted into asset
    // components, as well as path being rewritten to not contain meta values.
    database.ensure("custom://lorem.txt?uppercase")?;

    while database.is_busy() {
        database.maintain()?;
    }

    // Accessing asset and its extracted meta data via shortened path.
    let handle = database.find("custom://lorem.txt").unwrap();
    let (contents, meta) = handle.access::<(&String, &Meta)>(&database);
    println!("Custom asset meta: {meta:?}");
    println!("Custom asset contents: {contents:?}");

    Ok(())
}

#[derive(Debug, Default)]
struct Meta {
    uppercase: bool,
}

struct CustomAssetProtocol;

impl AssetProtocol for CustomAssetProtocol {
    fn name(&self) -> &str {
        "custom"
    }

    // Allows to extract initial configuration from asset path meta data.
    fn extract_bundle_from_path(&self, path: &AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let mut meta = Meta::default();
        if path.has_meta_key("uppercase") {
            meta.uppercase = true;
        }
        Ok(DynamicBundle::new(meta).unwrap())
    }

    // Allows to remove meta values from spawned asset path.
    fn rewrite_path(&self, path: AssetPathStatic) -> Result<AssetPathStatic, Box<dyn Error>> {
        Ok(AssetPathStatic::from_parts(
            path.protocol(),
            path.path(),
            "",
        ))
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let uppercase = storage.component::<true, Meta>(handle.entity())?.uppercase;
        let mut contents = String::from_utf8(bytes)?;
        if uppercase {
            contents = contents.to_uppercase();
        }
        storage.insert(handle.entity(), (contents,))?;
        Ok(())
    }
}
