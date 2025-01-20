# Asset Fetch

Asset fetch engines are units that implement `AssetFetch` trait that tells asset
manager how and where from to get requested asset bytes. This covers first step
of asset progression, where next step uses asset protocol to decode loaded bytes.

Usually what we can see in asset management libraries is asset loaders, which are
combination of loading bytes (from some specific to them source) and decoding
bytes into asset object. I've found this approach unnecessarily fixed and forcing
requirement for either implementing same asset loader for every asset source, or
to make all possible asset sources implemented in asset loader.

In `Keket` i went with decoupling bytes loading from bytes decoding, so that user
can for example use different bytes source for different build mode or different
platform, without having to bytes decoding to care about where bytes come from.

```rust,ignore
{{#rustdoc_include ../../../crates/_/examples/01_hello_world.rs:fetch_use}}
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    // Whichever asset source we use, we will make them async load.
    .with_fetch(DeferredAssetFetch::new(
        // Hot-reloading assets from file system for development.
        #[cfg(not(feature = "shipping"))]
        HotReloadFileAssetFetch::new(
            FileAssetFetch::default().with_root("assets"),
            Duration::from_secs(5),
        )?
        // Loading assets from asset pack REDB database.
        #[cfg(feature = "shipping")]
        ContainerAssetFetch::new(RedbContainerPartialFetch::new(
            Database::create("./assets.redb")?,
            "assets",
        ))
    ));

let handle = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", handle.access::<&String>(&database));
```

And this is how easy it is to implement new asset fetch engine:

```rust,ignore
impl AssetFetch for FileAssetFetch {
    fn load_bytes(&self, path: AssetPath) -> Result<DynamicBundle, Box<dyn Error>> {
        let file_path = self.root.join(path.path());
        let bytes = std::fs::read(&file_path)
            .map_err(|error| format!("Failed to load `{:?}` file bytes: {}", file_path, error))?;
        let mut bundle = DynamicBundle::default();
        bundle
            .add_component(AssetBytesAreReadyToProcess(bytes))
            .ok()
            .unwrap();
        bundle.add_component(AssetFromFile).ok().unwrap();
        bundle
            .add_component(std::fs::metadata(&file_path)?)
            .ok()
            .unwrap();
        bundle.add_component(file_path).ok().unwrap();
        Ok(bundle)
    }
}
```

> `AssetBytesAreReadyToProcess` component is crucial in asset progression, because
> it marks asset for database (and outside systems) as loaded but not yet decoded,
> so database can detect these assets and trigger asset decoding with their protocol.

File asset fetch also adds other components such as:

- `AssetFromFile` - tag component that allows to query all assets from file system.
- `std::fs::Metadata` - file metadata that can be used for example for size stats.
- `PathBuf` - file system path that can be used for stuff like hot reloading.

Asset fetch engines other than `FileassetFetch` also do add their own custom
metadata to asset.

It's worth also to know that asset database uses stack of asset fetch engines,
to allow changing source of assets for particular code paths - we can push, pop,
swap or use-in-place fetch engines. Typical scenario of that would be to
procedurally create an asset container that we can then use as container source
for future assets.
