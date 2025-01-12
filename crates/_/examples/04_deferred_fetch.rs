use keket::{
    database::AssetDatabase,
    fetch::{deferred::DeferredAssetFetch, file::FileAssetFetch},
    protocol::bytes::BytesAssetProtocol,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(BytesAssetProtocol)
        // Deferred asset fetch runs fetching jobs in threads.
        .with_fetch(DeferredAssetFetch::new(
            // File asset fetch implements deferred job mechanism.
            FileAssetFetch::default().with_root("resources"),
        ));

    let package = database.ensure("bytes://package.zip")?;

    // Simulate waiting for asset bytes loading to complete.
    while package.awaits_deferred_job(&database) {
        println!("Package awaits deferred job done");
        database.maintain()?;
    }

    // Run another maintain pass to process loaded bytes.
    database.maintain()?;

    println!(
        "Package byte size: {}",
        package.access::<&Vec<u8>>(&database).len()
    );

    Ok(())
}
