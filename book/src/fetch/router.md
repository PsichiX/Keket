# Router fetch wrapper

`RouterAssetFetch` allows to combine multiple fetch engines into routes, so that
specific fetch engine to use for given asset is decided by the pattern in asset
path - this is useful when we might want to have main game assets and additional
DLC/mod asset sources.

```rust,ignore
let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_protocol(BytesAssetProtocol)
    .with_fetch(
        RouterAssetFetch::default()
            .route(
                // Main assets source.
                RouterPattern::default(),
                ContainerAssetFetch::new(
                    ZipContainerPartialFetch::new(ZipArchive::new(
                        File::open("./resources/main.zip")?,
                    )?)
                ),
            )
            .route(
                // DLC assets source, with higher priority pattern to try match
                // before main assets route matches.
                RouterPattern::new("dlc/").priority(1),
                ContainerAssetFetch::new(
                    ZipContainerPartialFetch::new(ZipArchive::new(
                        File::open("./resources/dlc.zip")?,
                    )?)
                ),
            ),
    );

let lorem = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

let trash = database.ensure("bytes://dlc/trash.bin")?;
println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));
```
