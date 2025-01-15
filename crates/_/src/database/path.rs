use crate::database::{handle::AssetHandle, AssetDatabase};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, error::Error, fmt::Write, ops::Range};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub struct AssetPath<'a> {
    content: Cow<'a, str>,
    #[serde(skip)]
    protocol: Range<usize>,
    #[serde(skip)]
    path: Range<usize>,
    #[serde(skip)]
    meta: Range<usize>,
}

impl<'a> AssetPath<'a> {
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        let content: Cow<'a, str> = content.into();
        let (protocol, path_start) = if let Some(index) = content.find("://") {
            (0..index, index + b"://".len())
        } else {
            (0..0, 0)
        };
        let (path_end, meta) = if let Some(path_end) = content.find('?') {
            (path_end, (path_end + b"?".len())..content.len())
        } else {
            (content.len(), content.len()..content.len())
        };
        Self {
            content,
            protocol,
            path: path_start..path_end,
            meta,
        }
    }

    pub fn from_parts(protocol: &str, path: &str, meta: &str) -> Self {
        let mut result = String::new();
        if !protocol.is_empty() {
            let _ = write!(&mut result, "{}://", protocol);
        }
        let _ = write!(&mut result, "{}", path);
        if !meta.is_empty() {
            let _ = write!(&mut result, "?{}", meta);
        }
        Self::new(result)
    }

    pub fn into_static(self) -> AssetPath<'static> {
        AssetPath {
            content: Cow::Owned(self.content.into_owned()),
            protocol: self.protocol,
            path: self.path,
            meta: self.meta,
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn protocol(&self) -> &str {
        &self.content[self.protocol.clone()]
    }

    pub fn path(&self) -> &str {
        &self.content[self.path.clone()]
    }

    pub fn path_parts(&self) -> impl Iterator<Item = &str> {
        self.path().split(&['/', '\\'])
    }

    pub fn meta(&self) -> &str {
        &self.content[self.meta.clone()]
    }

    pub fn meta_items(&self) -> impl Iterator<Item = (&str, &str)> {
        self.meta()
            .split("&")
            .filter(|part| !part.is_empty())
            .map(|part| {
                if let Some(index) = part.find("=") {
                    (part[..index].trim(), part[(index + b"=".len())..].trim())
                } else {
                    (part.trim(), "")
                }
            })
    }

    pub fn try_meta(&self) -> Option<&str> {
        let meta = self.meta();
        if meta.is_empty() {
            None
        } else {
            Some(meta)
        }
    }

    pub fn path_with_meta(&self) -> &str {
        &self.content[self.path.start..self.meta.end]
    }

    pub fn resolve(&self, database: &mut AssetDatabase) -> Result<AssetHandle, Box<dyn Error>> {
        database.ensure(self.clone().into_static())
    }
}

impl std::fmt::Display for AssetPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.protocol.is_empty() {
            write!(f, "{}://", self.protocol())?;
        }
        write!(f, "{}", self.path())?;
        if !self.meta.is_empty() {
            write!(f, "?{}", self.meta())?;
        }
        Ok(())
    }
}

impl<'a> From<&'a str> for AssetPath<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AssetPath<'_> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<AssetPath<'_>> for String {
    fn from(val: AssetPath<'_>) -> Self {
        val.content.into()
    }
}

impl<'a> From<Cow<'a, str>> for AssetPath<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::new(value)
    }
}
