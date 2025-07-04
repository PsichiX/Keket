use keket::{
    database::{AssetDatabase, inspector::AssetInspector},
    fetch::file::FileAssetFetch,
    protocol::bundle::BundleAssetProtocol,
    store::file::FileAssetStore,
};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct Person {
    name: String,
    age: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(BundleAssetProtocol::new(
            "json",
            (
                |bytes: Vec<u8>| {
                    let asset = serde_json::from_slice::<Person>(&bytes)?;
                    Ok((asset,).into())
                },
                // Additionally to decoding asset we can also encode it back to bytes.
                // This is useful for saving assets to storage.
                |inspector: AssetInspector| {
                    let asset = inspector.access::<&Person>();
                    let bytes = serde_json::to_vec(asset)?;
                    Ok(bytes.into())
                },
            ),
        ))
        .with_fetch(FileAssetFetch::default().with_root("resources"))
        .with_store(FileAssetStore::default().with_root("resources"));

    let _ = std::fs::remove_file("./resources/saved.json");

    let before = database.spawn(
        "json://saved.json",
        (Person {
            name: "Alice".to_owned(),
            age: 42,
        },),
    )?;
    println!("Before: {:?}", before.access::<&Person>(&database));
    before.store(&mut database)?;

    while database.is_busy() {
        database.maintain()?;
    }

    before.delete(&mut database);
    assert!(!before.does_exists(&database));

    // Load the asset from storage, we get previously saved asset content.
    let after = database.ensure("json://saved.json")?;
    println!("After: {:?}", after.access::<&Person>(&database));
    /* ANCHOR_END: main */

    Ok(())
}
