# Deferred loading wrapper

`DeferredAssetFetch` allows to asynchronously load requested bytes in background
thread(s). It is encouraged to use it for heavy latency fetch engines like file
system or network fetch engines.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(BytesAssetProtocol)
    .with_fetch(DeferredAssetFetch::new(
        FileAssetFetch::default().with_root("assets"),
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
```
