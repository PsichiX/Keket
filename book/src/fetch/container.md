# Assets container wrapper

`ContainerAssetFetch` allows for partial loading from sources that are considered
containers that store other assets. Typical container example would be ZIP archives,
databases, etc.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/02_zip.rs:zip}}
```

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/02_zip.rs:main}}
```
