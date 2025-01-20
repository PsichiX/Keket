# File system

`FileAssetFetch` is a simple fetch engine that reads entire bytes of an asset
file synchronously.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/01_hello_world.rs:main}}
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run file system fetching jobs in background.
