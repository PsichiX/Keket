use keket::{
    database::AssetDatabase,
    fetch::{fallback::FallbackAssetFetch, file::FileAssetFetch},
    protocol::text::TextAssetProtocol,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_fetch(
            // Fallback asset fetch in case of error on requested asset bytes load
            // will try to load asset with matching protocol from fallback paths.
            FallbackAssetFetch::new(FileAssetFetch::default().with_root("resources"))
                // This fallback asset does not exists so it will be ignored.
                .path("text://this-fails-to-load.txt")
                // This asset exists so it will be loaded as fallback.
                .path("text://lorem.txt"),
        );

    // This asset exists so it loads normally.
    let lorem = database.ensure("text://lorem.txt")?;

    // This asset does not exists so it loads fallback asset.
    let non_existent = database.ensure("text://non-existent.txt")?;

    if lorem.access::<&String>(&database) == non_existent.access::<&String>(&database) {
        println!("Non existent asset loaded from fallback!");
    }
    /* ANCHOR_END: main */

    Ok(())
}
