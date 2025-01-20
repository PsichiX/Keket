# Asset Handle

`AssetHandle` is a thin wrapper around ECS entity type (generational index).
Its used as key to asset in database, to allow said database to perform operations
on the asset using that handle - handles are returned when asset gets spawned in
storage, so every spawned handle always point to existing asset (unless asset gets
unloaded/deleted).

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/01_hello_world.rs:handle}}
```
