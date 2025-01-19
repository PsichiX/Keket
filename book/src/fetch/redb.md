# REDB database

> **Required crate: `keket-redb`.**

`RedbContainerPartialFetch` partial fetch engine allows to unpack asset bytes
from REDB database - useful for asset packs.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_fetch(ContainerAssetFetch::new(RedbContainerPartialFetch::new(
        Database::create("./assets/database.redb")?,
        "assets",
    )));

let lorem = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run REDB fetching jobs in background.
