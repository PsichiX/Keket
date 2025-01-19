# Group

`GroupAssetProtocol` allows to load group of assets at once as its dependencies.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_protocol(BytesAssetProtocol)
    .with_protocol(GroupAssetProtocol)
    .with_fetch(FileAssetFetch::default().with_root("assets"));

let group = database.ensure("group://group.txt")?;

while database.is_busy() {
    database.maintain()?;
}

let lorem = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

let trash = database.ensure("bytes://trash.bin")?;
println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

for dependency in group.dependencies(&database) {
    println!(
        "Group dependency: {}",
        dependency.access::<&AssetPath>(&database)
    );
}
```
