use keket::{
    database::{reference::SmartAssetRef, AssetDatabase},
    fetch::file::FileAssetFetch,
    protocol::text::TextAssetProtocol,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_fetch(FileAssetFetch::default().with_root("resources"));

    /* ANCHOR: smart_reference */
    let lorem = SmartAssetRef::ensured("text://lorem.txt", &mut database)?;
    println!(
        "Lorem Ipsum: {}",
        lorem.resolve(&database)?.access::<&String>()
    );

    assert_eq!(database.storage.len(), 1);
    drop(lorem);
    database.maintain()?;
    assert_eq!(database.storage.len(), 0);
    /* ANCHOR_END: smart_reference */
    /* ANCHOR_END: main */

    Ok(())
}
