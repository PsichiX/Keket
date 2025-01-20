# Asset Protocol

Asset protocols are units that decode bytes and then does some things with decoded
data, usually put components into asset, schedule dependencies to load and relate
them with source asset, if there are any dependencies.

Asset protocols are second step in asset progression - they are decoupled from
asset fetch engines to make them not care about source of the asset and by that
be used no matter where assets come from, which solves the need for asset loaders
to implement with all possible asset sources, which is cumbersome to deal with.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/01_hello_world.rs:main}}
```

For type to be considered asset protocol, it must implement `AssetProtocol` trait
that has methods for processing asset bytes:

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/11_custom_protocol_simple.rs:custom_asset_protocol}}
```

Optionally type can implement more specific asset processing that doesn't assume
you only care about bytes or even only processed asset:

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/12_custom_protocol_advanced.rs:custom_asset_protocol}}
```
