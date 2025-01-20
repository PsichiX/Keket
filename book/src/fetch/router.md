# Router fetch wrapper

`RouterAssetFetch` allows to combine multiple fetch engines into routes, so that
specific fetch engine to use for given asset is decided by the pattern in asset
path - this is useful when we might want to have main game assets and additional
DLC/mod asset sources.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/06_router_fetch.rs:main}}
```
