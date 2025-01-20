# Fallback assets wrapper

`FallbackAssetFetch` allows to provide list of assets that should be considered
to load instead of requested assets (with matching protocol) when requested asset
fails to load - for example if requested texture does not exists, we can fallback
to popular magenta solid color texture that indicates missing textures.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/07_fallback.rs:main}}
```
