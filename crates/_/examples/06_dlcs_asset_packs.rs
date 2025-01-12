use keket::{
    database::{path::AssetPath, AssetDatabase},
    fetch::{
        container::{ContainerAssetFetch, ContainerPartialFetch},
        router::{RouterAssetFetch, RouterPattern},
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
                    RouterPattern::default(),
                    ContainerAssetFetch::new(ZipContainerPartialFetch::new(ZipArchive::new(
                        File::open("./resources/main.zip")?,
                    )?)),
                )
                .route(
                    RouterPattern::new("dlc/"),
                    ContainerAssetFetch::new(ZipContainerPartialFetch::new(ZipArchive::new(
                        File::open("./resources/dlc.zip")?,
                    )?)),
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
    fn part(&mut self, path: AssetPath) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut file = self
            .archive
            .by_name(path.path())
            .map_err(|error| format!("Could not read zip file: `{}` - {}", path.path(), error))?;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}
