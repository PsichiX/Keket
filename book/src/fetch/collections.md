# In-memory collections

In rare cases we need to have assets in-memory - embeded platforms, or just assets
baked into binary (typically simple web wasm games) - we can use collections to
store paths and their bytes and allow assets to be fetched from there.

```rust,ignore
const ASSETS: &[(&str, &[u8])] = &[
    ("lorem.txt", include_bytes!("./assets/lorem.txt")),
    ("trash.bin", include_bytes!("./assets/trash.bin")),
];

let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_protocol(BytesAssetProtocol)
    .with_fetch(ASSETS);

let lorem = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

let trash = database.ensure("bytes://trash.bin")?;
println!("Trash: {:?}", bytes.access::<&Vec<u8>>(&database));
```
