# Fallback assets wrapper

`FallbackAssetFetch` allows to provide list of assets that should be considered
to load instead of requested assets (with matching protocol) when requested asset
fails to load - for example if requested texture does not exists, we can fallback
to popular magenta solid color texture that indicates missing textures.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_fetch(
        FallbackAssetFetch::new(FileAssetFetch::default().with_root("assets"))
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
```
