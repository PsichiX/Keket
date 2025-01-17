use std::{borrow::Cow, collections::HashSet};

/// A structure to manage a collection of unique tags associated with assets.
///
/// `AssetTags` provides a set-based implementation to store and query tags.
/// The tags are stored as `Cow<'static, str>`, allowing both borrowed and owned strings.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetTags {
    tags: HashSet<Cow<'static, str>>,
}

impl AssetTags {
    /// Creates a new `AssetTags` with a single initial tag.
    ///
    /// # Arguments
    /// - `tag`: A string-like value to initialize the tag set.
    ///
    /// # Returns
    /// A new instance of `AssetTags` containing the specified tag.
    pub fn new(tag: impl Into<Cow<'static, str>>) -> Self {
        let mut tags = HashSet::with_capacity(1);
        tags.insert(tag.into());
        Self { tags }
    }

    /// Adds a new tag to the `AssetTags` and returns the updated instance.
    ///
    /// # Arguments
    /// - `tag`: The tag to be added.
    ///
    /// # Returns
    /// A new `AssetTags` instance with the additional tag.
    pub fn with(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.add(tag);
        self
    }

    /// Returns the number of tags in the collection.
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// Checks if the tag collection is empty.
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// Checks if a specific tag is present in the collection.
    ///
    /// # Arguments
    /// - `tag`: The tag to check.
    ///
    /// # Returns
    /// `true` if the tag is found, `false` otherwise.
    pub fn has(&mut self, tag: &str) {
        self.tags.contains(tag);
    }

    /// Adds a new tag to the collection.
    ///
    /// # Arguments
    /// - `tag`: The tag to be added.
    pub fn add(&mut self, tag: impl Into<Cow<'static, str>>) {
        self.tags.insert(tag.into());
    }

    /// Removes a tag from the collection.
    ///
    /// # Arguments
    /// - `tag`: The tag to be removed.
    pub fn remove(&mut self, tag: &str) {
        self.tags.remove(tag);
    }

    /// Returns an iterator over the tags in the collection.
    ///
    /// # Returns
    /// An iterator yielding each tag as a string slice.
    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.tags.iter().map(|tag| tag.as_ref())
    }

    /// Checks if this collection is a subset of another `AssetTags` collection.
    ///
    /// # Arguments
    /// - `other`: The other `AssetTags` collection to compare against.
    ///
    /// # Returns
    /// `true` if all tags in this collection are also present in the other collection.
    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.tags.is_subset(&other.tags)
    }

    /// Checks if this collection is a superset of another `AssetTags` collection.
    ///
    /// # Arguments
    /// - `other`: The other `AssetTags` collection to compare against.
    ///
    /// # Returns
    /// `true` if all tags in the other collection are also present in this collection.
    pub fn is_superset_of(&self, other: &Self) -> bool {
        self.tags.is_superset(&other.tags)
    }

    /// Creates a new `AssetTags` collection with tags that exist in both this and another collection.
    ///
    /// # Arguments
    /// - `other`: The other `AssetTags` collection to intersect with.
    ///
    /// # Returns
    /// A new `AssetTags` containing the common tags.
    pub fn intersection(&self, other: &Self) -> Self {
        self.tags.intersection(&other.tags).cloned().collect()
    }

    /// Creates a new `AssetTags` collection with tags that exist in either this or another collection.
    ///
    /// # Arguments
    /// - `other`: The other `AssetTags` collection to union with.
    ///
    /// # Returns
    /// A new `AssetTags` containing all unique tags from both collections.
    pub fn union(&self, other: &Self) -> Self {
        self.tags.union(&other.tags).cloned().collect()
    }
}

impl FromIterator<Cow<'static, str>> for AssetTags {
    fn from_iter<T: IntoIterator<Item = Cow<'static, str>>>(iter: T) -> Self {
        Self {
            tags: iter.into_iter().collect(),
        }
    }
}
