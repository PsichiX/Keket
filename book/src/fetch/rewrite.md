# Rewrite asset path wrapper

`RewriteAssetFetch` allows to rewrite requested asset path to some other.
This is useful for scenarios like localized assets or assets versioning, where
there might be different versions of assets based on some runtime state.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/15_localized_assets.rs:main}}
```

> Rewritten asset paths do not change path in asset entity - this is deliberate
> design decision to make outside systems not care about possible asset path
> change when trying to resolve asset handle by its path.
