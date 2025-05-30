use keket::{
    database::{AssetDatabase, path::AssetPath},
    fetch::container::{ContainerAssetFetch, ContainerPartialFetch},
    protocol::text::TextAssetProtocol,
};
use std::{error::Error, fs::File, io::Read};
use zip::ZipArchive;

fn main() -> Result<(), Box<dyn Error>> {
    /* ANCHOR: main */
    let mut database = AssetDatabase::default()
        .with_protocol(TextAssetProtocol)
        // Container asset fetch allows to use partial asset fetch object
        // that can take asset path and returns bytes from some container.
        .with_fetch(ContainerAssetFetch::new(ZipContainerPartialFetch::new(
            ZipArchive::new(File::open("./resources/package.zip")?)?,
        )));

    let lorem = database.ensure("text://lorem.txt")?;
    println!("Lorem Ipsum: {}", lorem.access::<&String>(&database));
    /* ANCHOR_END: main */

    Ok(())
}

/* ANCHOR: zip */
// Here we show how to make ZIP archive file reader.
struct ZipContainerPartialFetch {
    archive: ZipArchive<File>,
}

impl ZipContainerPartialFetch {
    pub fn new(archive: ZipArchive<File>) -> Self {
        Self { archive }
    }
}

impl ContainerPartialFetch for ZipContainerPartialFetch {
    // We use input path and try to unpack file under that path from ZIP archive.
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
/* ANCHOR_END: zip */
