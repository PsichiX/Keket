use crate::database::{AssetDatabase, handle::AssetHandle};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    error::Error,
    fmt::Write,
    hash::{Hash, Hasher},
    ops::Range,
};

/// A static version of `AssetPath` that has a `'static` lifetime.
pub type AssetPathStatic = AssetPath<'static>;

/// Represents an asset path, including its protocol, path, and optional metadata.
///
/// # Structure
/// The `AssetPath` is divided into three main components:
/// - **Protocol**: The scheme of the asset path (e.g., `file`, `http`).
/// - **Path**: The main path to the asset (e.g., `/assets/texture.png`).
/// - **Meta**: Optional metadata for the asset, typically a query string (e.g., `?version=1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub struct AssetPath<'a> {
    /// The complete asset path content.
    content: Cow<'a, str>,
    /// Range of the protocol in the content string.
    #[serde(skip)]
    protocol: Range<usize>,
    /// Range of the path in the content string.
    #[serde(skip)]
    path: Range<usize>,
    /// Range of the meta section in the content string.
    #[serde(skip)]
    meta: Range<usize>,
}

impl<'a> AssetPath<'a> {
    /// Creates a new `AssetPath` from the given content.
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

    /// Constructs an `AssetPath` from separate protocol, path, and metadata strings.
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

    /// Converts the `AssetPath` into a static version, consuming the current instance.
    pub fn into_static(self) -> AssetPathStatic {
        AssetPath {
            content: Cow::Owned(self.content.into_owned()),
            protocol: self.protocol,
            path: self.path,
            meta: self.meta,
        }
    }

    /// Retrieves the full content of the `AssetPath`.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns the protocol part of the `AssetPath`.
    pub fn protocol(&self) -> &str {
        &self.content[self.protocol.clone()]
    }

    /// Returns the path part of the `AssetPath`.
    pub fn path(&self) -> &str {
        &self.content[self.path.clone()]
    }

    /// Returns the path part extension of the `AssetPath`.
    pub fn path_extension(&self) -> Option<&str> {
        let path = self.path();
        path.rfind('.').map(|index| &path[(index + b".".len())..])
    }

    /// Returns the path part extension with preceding dot of the `AssetPath`.
    pub fn path_dot_extension(&self) -> Option<&str> {
        let path = self.path();
        path.rfind('.').map(|index| &path[index..])
    }

    /// Returns the path part without extension of the `AssetPath`.
    pub fn path_without_extension(&self) -> &str {
        let path = self.path();
        path.rfind('.').map(|index| &path[..index]).unwrap_or(path)
    }

    /// Splits the path into its component parts.
    pub fn path_parts(&self) -> impl Iterator<Item = &str> {
        self.path().split(&['/', '\\'])
    }

    /// Returns the metadata part of the `AssetPath`.
    pub fn meta(&self) -> &str {
        &self.content[self.meta.clone()]
    }

    /// Parses the metadata into key-value pairs.
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

    /// Checks if path has specific meta key.
    pub fn has_meta_key(&self, key: &str) -> bool {
        self.meta_items().any(|(k, _)| k == key)
    }

    /// Checks if path has specific meta key-value.
    pub fn has_meta_key_value(&self, key: &str, value: &str) -> bool {
        self.meta_items().any(|(k, v)| k == key && v == value)
    }

    /// Tries to retrieve the metadata, returning `None` if it's empty.
    pub fn try_meta(&self) -> Option<&str> {
        let meta = self.meta();
        if meta.is_empty() { None } else { Some(meta) }
    }

    /// Returns the combined path and metadata of the `AssetPath`.
    pub fn path_with_meta(&self) -> &str {
        &self.content[self.path.start..self.meta.end]
    }

    /// Schedules the asset in the given `AssetDatabase`.
    pub fn schedule(&self, database: &mut AssetDatabase) -> Result<AssetHandle, Box<dyn Error>> {
        database.schedule(self.clone().into_static())
    }

    /// Ensures the asset is loaded in the given `AssetDatabase`.
    pub fn ensure(&self, database: &mut AssetDatabase) -> Result<AssetHandle, Box<dyn Error>> {
        database.ensure(self.clone().into_static())
    }

    /// Searches for the asset in the given `AssetDatabase`.
    pub fn find(&self, database: &AssetDatabase) -> Option<AssetHandle> {
        database.find(self.clone().into_static())
    }
}

impl Hash for AssetPath<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content.hash(state);
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
