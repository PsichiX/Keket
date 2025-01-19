# HTTP requests

> **Required crate: `keket-http`.**

`HttpAssetFetch` allows to fetch bytes with HTTP sources.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    // HTTP asset fetch with root URL for assets.
    .with_fetch(DeferredAssetFetch::new(HttpAssetFetch::new(
        "https://raw.githubusercontent.com/PsichiX/Keket/refs/heads/master/resources/",
    )?));

// Ensure assets exists or start getting fetched.
let lorem = database.ensure("text://lorem.txt")?;

// Wait for pending fetches.
while database.does_await_deferred_job() {
    database.maintain()?;
}

// After all assets bytes are fetched, process them into assets.
database.maintain()?;

println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run HTTP fetching jobs in background.
