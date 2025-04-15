use keket::{
    database::{AssetDatabase, AssetReferenceCounter, reference::SmartAssetRef},
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
    // Load asset and mark its first reference.
    let first = SmartAssetRef::new("text://lorem.txt", &mut database)?;
    println!(
        "Lorem Ipsum: {}",
        first.resolve(&database)?.access::<&String>()
    );
    database.maintain()?;
    assert_eq!(database.storage.len(), 1);
    assert_eq!(
        first
            .resolve(&database)?
            .access::<&AssetReferenceCounter>()
            .counter(),
        1
    );

    // Clone the asset and mark its second reference.
    let second = first.clone();
    database.maintain()?;
    assert_eq!(database.storage.len(), 1);
    assert_eq!(
        second
            .resolve(&database)?
            .access::<&AssetReferenceCounter>()
            .counter(),
        2
    );

    // Ensure asset again from scratch and mark its third reference.
    let third = SmartAssetRef::new("text://lorem.txt", &mut database)?;
    database.maintain()?;
    assert_eq!(database.storage.len(), 1);
    assert_eq!(
        third
            .resolve(&database)?
            .access::<&AssetReferenceCounter>()
            .counter(),
        3
    );

    // Drop the first reference and expect 2 references.
    drop(first);
    database.maintain()?;
    assert_eq!(database.storage.len(), 1);
    assert_eq!(
        third
            .resolve(&database)?
            .access::<&AssetReferenceCounter>()
            .counter(),
        2
    );

    // Drop the second reference and expect 1 reference.
    drop(second);
    database.maintain()?;
    assert_eq!(database.storage.len(), 1);
    assert_eq!(
        third
            .resolve(&database)?
            .access::<&AssetReferenceCounter>()
            .counter(),
        1
    );

    // Drop the third reference and expect no references and asset no more existing.
    drop(third);
    database.maintain()?;
    assert_eq!(database.storage.len(), 0);
    /* ANCHOR_END: smart_reference */
    /* ANCHOR_END: main */

    Ok(())
}
