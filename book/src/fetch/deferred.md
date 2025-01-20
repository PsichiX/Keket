# Deferred loading wrapper

`DeferredAssetFetch` allows to asynchronously load requested bytes in background
thread(s). It is encouraged to use it for heavy latency fetch engines like file
system or network fetch engines.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/05_deferred_fetch.rs:main}}
```
