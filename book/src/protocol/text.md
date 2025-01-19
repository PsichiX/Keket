# Text

`TextAssetProtocol` allows to decode fetched bytes into `String` component.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_fetch(FileAssetFetch::default().with_root("assets"));

let lorem = database.ensure("text://lorem.txt")?;
println!("Text: {:?}", lorem.access::<&String>(&database));
```
