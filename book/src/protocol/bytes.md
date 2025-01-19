# Bytes

`BytesAssetProtocol` allows to just unwrap fetched bytes into `Vec<u8>` component.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(BytesAssetProtocol)
    .with_fetch(FileAssetFetch::default().with_root("assets"));

let trash = database.ensure("bytes://trash.bin")?;
println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
```
