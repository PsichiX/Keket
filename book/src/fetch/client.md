# Asset Server

> **Required crate: `keket-client`.**

`ClientAssetFetch` allows to get bytes from `Keket` Asset Server (`keket-server`
binary crate) - a humble beginnings to DDC infrastructure for live development.

```rust,ignore
{{#rustdoc_include ../../../crates/client/examples/hello_client.rs:main}}
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run Asset Server fetching jobs in background.
