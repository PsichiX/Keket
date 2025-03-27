# Asset Reference

`AssetRef` is a thin wrapper around asset path and optional cached asset handle.

The goal here was to allow to reduce resolving asset handles to one resolution
for given asset reference - searching for asset handle by asset path is unnecessary
work to do everytime we point to an asset by path and need to perform operations
on its handle, so with asset reference we can resolve asset handle once and reuse
its cached handle for future database operations.

Additional justification for asset references is serialization - this is preferred
way to reference assets in serializable data, so once container gets deserializaed
into typed data, asset handle resolution will happen only at first time.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/10_references.rs:custom_asset}}
```

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/10_references.rs:main}}
```

`SmartAssetRef` is a thin wrapper arount asset reference, that uses database
asset reference counting feature to automatically handle asset lifetime and
despawn asset when counter drops to 0.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/17_smart_references.rs:main}}
```
