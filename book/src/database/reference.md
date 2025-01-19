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
#[derive(Debug, Default, Deserialize)]
struct CustomAsset {
    content: String,
    next: AssetRef,
}

// Here we grab first asset by path.
let part1 = database.ensure("custom://part1.json")?;
let part1 = part1.access::<&CustomAsset>(&database);
println!("Part 1 content: {:?}", part1.content);

// Then we grab next asset by reference stored in loaded first asset.
// It's important to note that dependency assets should be scheduled to load by
// asset protocols that load parent asset in their processing method, otherwise
// dependent assets won't be resolved - here we assume custom asset protocol did
// loaded dependent assets so they can be resolved.
let part2 = part1.next.resolve(&database)?;
let part2 = part2.access::<&CustomAsset>();
println!("Part 2 content: {:?}", part2.content);
```
