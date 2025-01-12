use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::deferred::{AssetAwaitsDeferredJob, DeferredAssetFetch},
    protocol::bytes::BytesAssetProtocol,
};
use keket_http::{third_party::reqwest::Url, HttpAssetFetch};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(BytesAssetProtocol)
        .with_fetch(DeferredAssetFetch::new(HttpAssetFetch::new(
            "https://picsum.photos/",
        )?));

    let image = database.ensure("bytes://128")?;

    while image
        .access_checked::<&AssetAwaitsDeferredJob>(&database)
        .is_some()
    {
        println!("Image asset awaits deferred job done");
        database.maintain()?;
    }

    database.maintain()?;

    println!("Byte size: {}", image.access::<&Vec<u8>>(&database).len());

    for (asset_path, url) in database.storage.query::<true, (&AssetPath, &Url)>() {
        println!("Asset: `{}` at url: `{}`", asset_path, url);
    }

    Ok(())
}
