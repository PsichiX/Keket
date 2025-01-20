# REDB database

> **Required crate: `keket-redb`.**

`RedbContainerPartialFetch` partial fetch engine allows to unpack asset bytes
from REDB database - useful for asset packs.

```rust,ignore
{{#rustdoc_include ../../../crates/redb/examples/hello_redb.rs:main}}
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run REDB fetching jobs in background.
