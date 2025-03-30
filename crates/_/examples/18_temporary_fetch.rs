use keket::{
    database::AssetDatabase,
    fetch::file::FileAssetFetch,
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        // Dummy empty asset fetch to start with.
        .with_fetch([]);

    // Temporarily use different asset fetch to load asset from file.
    let lorem = database.using_fetch(
        FileAssetFetch::default().with_root("resources"),
        |database| database.ensure("text://lorem.txt"),
    )?;

    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
    /* ANCHOR_END: main */

    Ok(())
}
