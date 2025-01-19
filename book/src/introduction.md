# Introduction

Welcome to `Keket`!

Modern, flexible, modular Asset Management library built on top of ECS as its
storage.

```rust,ignore
let mut database = AssetDatabase::default()
    // Asset protocols tell how to deserialize bytes into assets.
    .with_protocol(TextAssetProtocol)
    .with_protocol(BytesAssetProtocol)
    // Bundle protocol allows for easly making a protocol that takes
    // bytes and returns bundle with optional dependencies.
    .with_protocol(BundleAssetProtocol::new("json", |bytes: Vec<u8>| {
        let asset = serde_json::from_slice::<Value>(&bytes)?;
        Ok((asset,).into())
    }))
    // Asset fetch tells how to get bytes from specific source.
    .with_fetch(FileAssetFetch::default().with_root("assets"));

// Ensure method either gives existing asset handle or creates new
// and loads asset if not existing yet in storage.
let lorem = database.ensure("text://lorem.txt")?;
// Accessing component(s) of asset entry.
// Assets can store multiple data associated to them, consider them meta data.
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

let json = database.ensure("json://person.json")?;
println!("JSON: {:#}", json.access::<&Value>(&database));

let trash = database.ensure("bytes://trash.bin")?;
println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

// We can query storage for asset components to process assets, just like with ECS.
for (asset_path, file_path, metadata) in database
    .storage
    .query::<true, (&AssetPath, &PathBuf, &Metadata)>()
{
    println!(
        "Asset: `{}` at location: {:?} has metadata: {:#?}",
        asset_path, file_path, metadata
    );
}
```

## Goal

`Keket` started as an experiment to tell how asset management would look like
with ECS as its storage, how we could make it play with modularity, what benefits
does ECS storage gives us.

Soon after first version got done, i've realized that it is quite powerful and
easily extendable, when we treat assets as entities with components as their data
(and meta data) and asset loaders (and systems outside of asset management) can
process and change assets in bulk the way they need, while not forcing any
particular closed specs structure on the assets themselves.
