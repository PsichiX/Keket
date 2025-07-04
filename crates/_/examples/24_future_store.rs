use keket::{
    database::{AssetDatabase, path::AssetPathStatic},
    fetch::file::FileAssetFetch,
    protocol::text::TextAssetProtocol,
    store::future::FutureAssetStore,
};
use std::{error::Error, path::PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"))
        .with_store(FutureAssetStore::new(tokio_save_file));

    let _ = tokio::fs::remove_file("./resources/saved2.txt").await;

    // Spawn a new asset.
    let before = database.spawn("text://saved2.txt", ("Abra cadabra!".to_owned(),))?;
    println!("Before: {}", before.access::<&String>(&database));
    // Request the asset to be stored using active asset store engine.
    before.store(&mut database)?;

    // Wait until the asset is stored.
    while database.is_busy() {
        database.maintain()?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Delete spawned asset from database just to show it will load from storage.
    before.delete(&mut database);
    assert!(!before.does_exists(&database));

    // Load the asset from storage, we get previously saved asset content.
    let after = database.ensure("text://saved2.txt")?;
    println!("After: {}", after.access::<&String>(&database));
    /* ANCHOR_END: main */

    Ok(())
}

/* ANCHOR: async_save_file */
async fn tokio_save_file(path: AssetPathStatic, bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let file_path = PathBuf::from("resources").join(path.path());

    tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    tokio::fs::write(&file_path, bytes).await?;

    Ok(())
}
/* ANCHOR_END: async_save_file */
