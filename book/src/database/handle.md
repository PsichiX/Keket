# Asset Handle

`AssetHandle` is a thin wrapper around ECS entity type (generational index).
Its used as key to asset in database, to allow said database to perform operations
on the asset using that handle - handles are returned when asset gets spawned in
storage, so every spawned handle always point to existing asset (unless asset gets
unloaded/deleted).

```rust,ignore
// `lorem` points to existing asset that was either already existing or was just
// spawned in storage.
let lorem = database.ensure("text://lorem.txt")?;
// Once we have valid handle, we can perform various operations on `lorem` asset
// (here we access its asset data immutably).
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

let json = database.ensure("json://person.json")?;
println!("JSON: {:#}", json.access::<&Value>(&database));

let trash = database.ensure("bytes://trash.bin")?;
println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
```
