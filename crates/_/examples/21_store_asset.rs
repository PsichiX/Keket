use keket::{
    database::AssetDatabase, fetch::file::FileAssetFetch, protocol::text::TextAssetProtocol,
    store::file::FileAssetStore,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"))
        // We can enable saving assets to storage using asset stores.
        // This one stores assets to the file system.
        .with_store(FileAssetStore::default().with_root("resources"));

    let _ = std::fs::remove_file("./resources/saved.txt");

    // Spawn a new asset.
    let before = database.spawn("text://saved.txt", ("Abra cadabra!".to_owned(),))?;
    println!("Before: {}", before.access::<&String>(&database));
    // Request the asset to be stored using active asset store engine.
    before.store(&mut database)?;

    // Wait until the asset is stored.
    while database.is_busy() {
        database.maintain()?;
    }

    // Delete spawned asset from database just to show it will load from storage.
    before.delete(&mut database);
    assert!(!before.does_exists(&database));

    // Load the asset from storage, we get previously saved asset content.
    let after = database.ensure("text://saved.txt")?;
    println!("After: {}", after.access::<&String>(&database));
    /* ANCHOR_END: main */

    Ok(())
}
