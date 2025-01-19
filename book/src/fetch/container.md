# Assets container wrapper

`ContainerAssetFetch` allows for partial loading from sources that are considered
containers that store other assets. Typical container example would be ZIP archives,
databases, etc.

```rust,ignore
struct ZipContainerPartialFetch {
    archive: ZipArchive<File>,
}

impl ZipContainerPartialFetch {
    pub fn new(archive: ZipArchive<File>) -> Self {
        Self { archive }
    }
}

impl ContainerPartialFetch for ZipContainerPartialFetch {
    fn load_bytes(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut file = self
            .archive
            .by_name(path.path())
            .map_err(|error| format!("Could not read zip file: `{}` - {}", path.path(), error))?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

let mut database = AssetDatabase::default()
    .with_protocol(TextAssetProtocol)
    .with_fetch(ContainerAssetFetch::new(ZipContainerPartialFetch::new(
        ZipArchive::new(File::open("./assets/package.zip")?)?,
    )));

let lorem = database.ensure("text://lorem.txt")?;
println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
```
