use keket::{
    database::AssetDatabase,
    fetch::{
        file::FileAssetFetch,
        throttled::{ThrottledAssetFetch, ThrottledAssetFetchStrategy},
    },
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        // Throttled asset fetch limits the number of concurrent fetches.
        .with_fetch(ThrottledAssetFetch::new(
            FileAssetFetch::default().with_root("resources"),
            // Here we set the strategy to allow only 1 fetch per maintenance tick.
            // You can also limit fetches by time duration per maintenance tick.
            ThrottledAssetFetchStrategy::Number(1),
        ));

    let lorem = database.ensure("text://lorem.txt")?;
    let trash = database.ensure("bytes://trash.bin")?;

    // We need to perform single maintenance tick to fetch first asset.
    database.maintain()?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    // Another maintenance tick to fetch second asset.
    database.maintain()?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
    /* ANCHOR_END: main */

    Ok(())
}
