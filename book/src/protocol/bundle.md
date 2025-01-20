# Bundle

`BundleAssetProtocol` allows to create easy custom bytes decoders.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/01_hello_world.rs:main}}
```

We can use closures or any type that implements `BundleWithDependenciesProcessor`
trait, which turns input bytes into `BundleWithDependencies` container with output
components bundle and optional list of dependencies to schedule after decoding.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/11_custom_protocol_simple.rs:custom_asset}}
```

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/11_custom_protocol_simple.rs:custom_asset_protocol}}
```
