# Async loading wrapper

`FutureAssetFetch` allows to asynchronously load requested bytes in async/await
futures. It's similar to `DeferredAssetFetch`, although it expects provided
custom async function that will handle loading asset bytes with custom
async/await compatible libraries like `tokio`.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/23_future_fetch.rs:main}}
```

And here is async function that handles loading bytes with `tokio`:

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/23_future_fetch.rs:async_read_file}}
```
