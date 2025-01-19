# Asset Server

> **Required crate: `keket-client`.**

`ClientAssetFetch` allows to get bytes from `Keket` Asset Server (`keket-server`
binary crate) - a humble beginnings to DDC infrastructure for live development.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_protocol(BytesAssetProtocol)
    // Client asset fetch to request files from asset server.
    .with_fetch(DeferredAssetFetch::new(ClientAssetFetch::new(
        // IP address of asset server we connect to.
        "127.0.0.1:8080",
    )?));

// Ensure assets exists or start getting fetched.
let lorem = database.ensure("text://lorem.txt")?;
let trash = database.ensure("bytes://trash.bin")?;

// Wait for pending fetches.
while database.does_await_deferred_job() {
    database.maintain()?;
}

// After all assets bytes are fetched, process them into assets.
database.maintain()?;

println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

println!("Listening for file changes...");
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
}
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run Asset Server fetching jobs in background.
