use std::{borrow::Cow, error::Error};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetPath<'a> {
    protocol: Cow<'a, str>,
    path: Cow<'a, str>,
    meta: Cow<'a, str>,
}

impl<'a> AssetPath<'a> {
    pub fn new(path: impl Into<Cow<'a, str>>) -> Result<Self, Box<dyn Error>> {
        let path: Cow<'a, str> = path.into();
        let path_str = path.as_ref();
        let Some(index) = path_str.find("://") else {
            return Err(format!("Asset path is missing protocol part: {:?}", path_str).into());
        };
        let protocol = &path_str[..index];
        let path_and_meta = &path_str[(index + "://".len())..];
        let (path, meta) = if let Some(index) = path_and_meta.find('?') {
            let meta = &path_and_meta[(index + 1)..];
            let path = &path_and_meta[..index];
            (path, meta)
        } else {
            (path_and_meta, "")
        };
        Ok(Self {
            protocol: Cow::Owned(protocol.to_string()),
            path: Cow::Owned(path.to_string()),
            meta: Cow::Owned(meta.to_string()),
        })
    }

    pub fn into_static(self) -> AssetPath<'static> {
        AssetPath {
            protocol: Cow::Owned(self.protocol.into_owned()),
            path: Cow::Owned(self.path.into_owned()),
            meta: Cow::Owned(self.meta.into_owned()),
        }
    }

    pub fn protocol(&self) -> &str {
        &self.protocol
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn meta(&self) -> &str {
        &self.meta
    }
}

impl std::fmt::Display for AssetPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.protocol, self.path)?;
        if !self.meta.is_empty() {
            write!(f, "?{}", self.meta)?;
        }
        Ok(())
    }
}

impl<'a> TryFrom<&'a str> for AssetPath<'a> {
    type Error = Box<dyn Error>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for AssetPath<'static> {
    type Error = Box<dyn Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'a> TryFrom<Cow<'a, str>> for AssetPath<'a> {
    type Error = Box<dyn Error>;

    fn try_from(value: Cow<'a, str>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
