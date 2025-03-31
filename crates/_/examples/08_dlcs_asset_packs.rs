use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::{
        container::{ContainerAssetFetch, ContainerPartialFetch},
        rewrite::RewriteAssetFetch,
        router::RouterAssetFetch,
    },
    protocol::{bytes::BytesAssetProtocol, text::TextAssetProtocol},
};
use std::{error::Error, fs::File, io::Read};
use zip::ZipArchive;

fn main() -> Result<(), Box<dyn Error>> {
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        .with_protocol(BytesAssetProtocol)
        .with_fetch(
            RouterAssetFetch::default()
                .route(
                    |_| true,
                    ContainerAssetFetch::new(ZipContainerPartialFetch::new(ZipArchive::new(
                        File::open("./resources/main.zip")?,
                    )?)),
                    0,
                )
                .route(
                    |path| path.path().starts_with("dlc/"),
                    RewriteAssetFetch::new(
                        ContainerAssetFetch::new(ZipContainerPartialFetch::new(ZipArchive::new(
                            File::open("./resources/dlc.zip")?,
                        )?)),
                        |path| {
                            Ok(format!(
                                "{}://{}",
                                path.protocol(),
                                path.path_with_meta().strip_prefix("dlc/").unwrap()
                            )
                            .into())
                        },
                    ),
                    1,
                ),
        );

    let lorem = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));

    let trash = database.ensure("bytes://dlc/trash.bin")?;
    println!("Bytes: {:?}", trash.access::<&Vec<u8>>(&database));

    Ok(())
}

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
