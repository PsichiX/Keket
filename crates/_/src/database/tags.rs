use std::{borrow::Cow, collections::HashSet};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AssetTags {
    tags: HashSet<Cow<'static, str>>,
}

impl AssetTags {
    pub fn new(tag: impl Into<Cow<'static, str>>) -> Self {
        let mut tags = HashSet::with_capacity(1);
        tags.insert(tag.into());
        Self { tags }
    }

    pub fn with(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.add(tag);
        self
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn has(&mut self, tag: &str) {
        self.tags.contains(tag);
    }

    pub fn add(&mut self, tag: impl Into<Cow<'static, str>>) {
        self.tags.insert(tag.into());
    }

    pub fn remove(&mut self, tag: &str) {
        self.tags.remove(tag);
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.tags.iter().map(|tag| tag.as_ref())
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        self.tags.is_subset(&other.tags)
    }

    pub fn is_superset_of(&self, other: &Self) -> bool {
        self.tags.is_superset(&other.tags)
    }

    pub fn intersection(&self, other: &Self) -> Self {
        self.tags.intersection(&other.tags).cloned().collect()
    }

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
