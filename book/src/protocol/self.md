# Asset Protocol

Asset protocols are units that decode bytes and then does some things with decoded
data, usually put components into asset, schedule dependencies to load and relate
them with source asset, if there are any dependencies.

Asset protocols are second step in asset progression - they are decoupled from
asset fetch engines to make them not care about source of the asset and by that
be used no matter where assets come from, which solves the need for asset loaders
to implement with all possible asset sources, which is cumbersome to deal with.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_fetch(FileAssetFetch::default().with_root("resources"));

let lorem = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
```

For type to be considered asset protocol, it must implement `AssetProtocol` trait
that has methods for processing asset bytes:

```rust,ignore
struct CustomAssetProtocol;

impl AssetProtocol for CustomAssetProtocol {
    fn name(&self) -> &str {
        "custom"
    }

    fn process_bytes(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
        bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        // Once we have asset bytes, we decode them into asset data.
        let asset = serde_json::from_slice::<CustomAsset>(&bytes)?;

        // We also have to extract dependencies if it has some.
        if let Some(path) = asset.next.clone() {
            // For every dependency we get, we need to spawn an asset entity
            // with AssetAwaitsResolution component to tell asset database
            // to start loading it.
            let entity = storage.spawn((path, AssetAwaitsResolution))?;

            // We should also relate processed asset with its dependency asset.
            // Dependency relations are useful for traversing asset graphs.
            storage.relate::<true, _>(AssetDependency, handle.entity(), entity)?;
        }

        Ok(())
    }
}
```

Optionally type can implement more specific asset processing that doesn't assume
you only care about bytes or even only processed asset:

```rust,ignore
impl AssetProtocol for CustomAssetProtocol {
    fn name(&self) -> &str {
        "custom"
    }

    fn process_asset(
        &mut self,
        handle: AssetHandle,
        storage: &mut World,
    ) -> Result<(), Box<dyn Error>> {
        // Always remember that in order for asset to be considered done with
        // processing bytes, it has to remove AssetBytesAreReadyToProcess
        // component from that asset. We are doing that by taking its bytes
        // content first and removing it after.
        let bytes = {
            let mut bytes = storage
                .component_mut::<true, AssetBytesAreReadyToProcess>(
                    handle.entity()
                )?;
            std::mem::take(&mut bytes.0)
        };
        storage.remove::<(AssetBytesAreReadyToProcess,)>(handle.entity())?;

        let asset = serde_json::from_slice::<CustomAsset>(&bytes)?;

        if let Some(path) = asset.next.clone() {
            let entity = storage.spawn((path, AssetAwaitsResolution))?;
            storage.relate::<true, _>(AssetDependency, handle.entity(), entity)?;
        }

        Ok(())
    }
}
```
