# File system

`FileAssetFetch` is a simple fetch engine that reads entire bytes of an asset
file synchronously.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_protocol(BytesAssetProtocol)
    .with_fetch(FileAssetFetch::default().with_root("assets"));

let lorem = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run file system fetching jobs in background.
