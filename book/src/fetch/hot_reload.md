# Hot-reloading wrapper for file system

`HotReloadAssetFetch` wrapper alows assets loaded from `FileAssetFetch` to be
watched and reloaded when source file changes.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_protocol(BytesAssetProtocol)
    .with_fetch(HotReloadFileAssetFetch::new(
        FileAssetFetch::default().with_root("assets"),
        // File system watcher polling interval.
        Duration::from_secs(5),
    )?);

// First we fill database with some assets, hot reload only
// cares about changes in files present in database.
database.ensure("text://lorem.txt")?;
database.ensure("bytes://trash.bin")?;

loop {
    database.maintain()?;

    // With storage change detection we can ask for asset entities
    // that their paths were updated (hot reloading updates them).
    if let Some(changes) = database.storage.updated() {
        for entity in changes.iter_of::<AssetPath>() {
            println!(
                "Asset changed: `{}`",
                AssetHandle::new(entity).access::<&AssetPath>(&database)
            );
        }
    }

    // Simulate systems that detect particular asset type reload.
    for entity in database.storage.added().iter_of::<String>() {
        println!(
            "Text asset changed: `{}`",
            AssetHandle::new(entity).access::<&String>(&database)
        );
    }
    for entity in database.storage.added().iter_of::<Vec<u8>>() {
        println!(
            "Bytes asset changed: `{:?}`",
            AssetHandle::new(entity).access::<&Vec<u8>>(&database)
        );
    }
}
```
