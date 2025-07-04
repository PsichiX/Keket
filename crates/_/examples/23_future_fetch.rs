use anput::bundle::DynamicBundle;
use keket::{
    database::{AssetDatabase, path::AssetPathStatic},
    fetch::{AssetBytesAreReadyToProcess, future::FutureAssetFetch},
    protocol::bytes::BytesAssetProtocol,
};
use std::{error::Error, path::PathBuf};

/* ANCHOR: async_read_file */
async fn tokio_load_file_bundle(path: AssetPathStatic) -> Result<DynamicBundle, Box<dyn Error>> {
    let file_path = PathBuf::from("resources").join(path.path());

    let bytes = tokio::fs::read(&file_path).await?;

    let mut bundle = DynamicBundle::default();
    bundle
        .add_component(AssetBytesAreReadyToProcess(bytes))
        .map_err(|_| format!("Failed to add bytes to bundle for asset: {}", path))?;
    Ok(bundle)
}
/* ANCHOR_END: async_read_file */

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(BytesAssetProtocol)
        // Future asset fetch uses async function to handle providing
        // asset bytes asynchronously with async/await.
        .with_fetch(FutureAssetFetch::new(tokio_load_file_bundle));

    let package = database.ensure("bytes://package.zip")?;

    // Run maintain passes to load and process loaded bytes.
    while !package.is_ready_to_use(&database) {
        database.maintain()?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    println!(
        "Package byte size: {}",
        package.access::<&Vec<u8>>(&database).len()
    );
    /* ANCHOR_END: main */

    Ok(())
}
