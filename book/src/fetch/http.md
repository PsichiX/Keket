# HTTP requests

> **Required crate: `keket-http`.**

`HttpAssetFetch` allows to fetch bytes with HTTP sources.

```rust,ignore
{{#rustdoc_include ../../../crates/http/examples/hello_http.rs:main}}
```

> Since this is blocking fetch, you might want to wrap it with `DeferredAssetFetch`
> to run HTTP fetching jobs in background.
