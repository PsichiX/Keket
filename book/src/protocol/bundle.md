# Bundle

`BundleAssetProtocol` allows to create easy custom bytes decoders.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(BundleAssetProtocol::new("json", |bytes: Vec<u8>| {
        let asset = serde_json::from_slice::<Value>(&bytes)?;
        Ok((asset,).into())
    }))
    .with_fetch(FileAssetFetch::default().with_root("assets"));

let json = database.ensure("json://person.json")?;
println!("JSON: {:#}", json.access::<&Value>(&database));
```

We can use closures or any type that implements `BundleWithDependenciesProcessor`
trait, which turns input bytes into `BundleWithDependencies` container with output
components bundle and optional list of dependencies to schedule after decoding.

```rust,ignore
// Custom asset type.
#[derive(Debug, Default, Deserialize)]
struct CustomAsset {
    // Main content.
    content: String,
    // Optional sibling asset (content continuation).
    #[serde(default)]
    next: Option<AssetPathStatic>,
}

struct CustomAssetProcessor;

impl BundleWithDependenciesProcessor for CustomAssetProcessor {
    // Bundle of asset components this asset processor produces
    // from processed asset.
    type Bundle = (CustomAsset,);

    fn process_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<BundleWithDependencies<Self::Bundle>, Box<dyn Error>> {
        let asset = serde_json::from_slice::<CustomAsset>(&bytes)?;
        let dependency = asset.next.clone();
        // Return bundled components and optional dependency
        // which will be additionally loaded.
        Ok(BundleWithDependencies::new((asset,)).maybe_dependency(dependency))
    }
}
```
