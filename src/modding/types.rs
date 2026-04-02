//! Register definitions (defs)

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    num::NonZeroU32,
    str::FromStr,
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone)]
pub struct Registry<T> {
    definitions: Vec<(DefPath, T)>,
    lookup: HashMap<DefPath, Id<T>>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    /// Registers a definition with the given path and returns its ID.
    /// If the definition already exists, it is replaced and the existing ID is returned.
    pub fn register(&mut self, path: DefPath, def: T) -> Option<Id<T>> {
        if let Some(id) = self.lookup.get(&path).copied() {
            self.definitions[id.index()].1 = def;
            return Some(id);
        }

        let id = Id::from_index(self.definitions.len());
        self.definitions.push((path.clone(), def));
        self.lookup.insert(path, id);

        Some(id)
    }

    pub fn clear(&mut self) {
        self.definitions.clear();
        self.lookup.clear();
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Looks up the id of the definition associated with the given path.
    pub fn lookup(&self, path: &str) -> Option<Id<T>> {
        self.lookup.get(path).copied()
    }

    /// Resolves the path of the definition associated with the given ID.
    pub fn resolve(&self, id: Id<T>) -> Option<&DefPath> {
        self.definitions.get(id.index()).map(|r| &r.0)
    }

    /// Retrieves the definition associated with the given ID.
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.definitions.get(id.index()).map(|r| &r.1)
    }

    /// Retrieves the definition associated with the given path.
    pub fn get_by_path(&self, path: &str) -> Option<&T> {
        self.lookup(path).and_then(|id| self.get(id))
    }

    pub fn contains(&self, id: Id<T>) -> bool {
        self.definitions.len() > id.index()
    }

    pub fn contains_path(&self, path: &str) -> bool {
        self.lookup.contains_key(path)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DefPath, &T)> {
        self.definitions.iter().map(|(p, t)| (p, t))
    }

    /// Order is guaranteed to be from lowest to highest id.
    pub fn iter_with_id(&self) -> impl Iterator<Item = (Id<T>, &DefPath, &T)> {
        self.definitions
            .iter()
            .enumerate()
            .map(|(i, (p, t))| (Id::from_index(i), p, t))
    }
}

impl<T: Debug> Debug for Registry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, path, definition) in self.iter_with_id() {
            writeln!(f, "{} {}: {:?}", id.get(), path, definition)?;
        }
        Ok(())
    }
}

impl<T: Display> Display for Registry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, path, definition) in self.iter_with_id() {
            writeln!(f, "{} {}: {}", id.get(), path, definition)?;
        }
        Ok(())
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            definitions: Vec::new(),
            lookup: HashMap::new(),
        }
    }
}

/// A id to a definition in a registry.
pub struct Id<T>(NonZeroU32, PhantomData<T>);

impl<T> Id<T> {
    pub const fn new(id: u32) -> Self {
        Self(NonZeroU32::new(id).unwrap(), PhantomData)
    }

    pub const fn from_index(id: usize) -> Self {
        Self(NonZeroU32::new(id as u32 + 1).unwrap(), PhantomData)
    }

    pub const fn get(&self) -> u32 {
        self.0.get()
    }

    pub const fn index(&self) -> usize {
        self.0.get() as usize - 1
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({})", self.0)
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// A newtype wrapper over a `String` that ensures the segment is valid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct DefPathSegment(String);

impl DefPathSegment {
    pub fn new(segment: &str) -> Option<Self> {
        is_valid_segment(segment).then(|| Self(segment.into()))
    }

    pub fn join(&self, other: DefPath) -> DefPath {
        DefPath(format!("{}::{}", self, other))
    }
}

impl Display for DefPathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for DefPathSegment {
    type Error = ();

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

impl FromStr for DefPathSegment {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(s).ok_or(())
    }
}

impl Serialize for DefPathSegment {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DefPathSegment {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).ok_or(()).map_err(|_| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &"a valid path segment")
        })
    }
}

/// A newtype wrapper over a `String` that ensures the path is valid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct DefPath(String);

impl DefPath {
    pub fn new(path: &str) -> Option<Self> {
        if !Self::is_valid_path(path) {
            return None;
        }
        Some(Self(path.into()))
    }

    pub fn new_qualified(path: &str) -> Option<Self> {
        if !Self::is_valid_qualified_path(path) {
            return None;
        }
        Some(Self(path.into()))
    }

    pub fn from_parts(
        namespace: impl TryInto<DefPath>,
        path: impl TryInto<DefPath>,
    ) -> Option<Self> {
        let namespace = namespace.try_into().ok()?;
        let path = path.try_into().ok()?;
        Self::new(&format!("{}::{}", namespace, path))
    }

    pub fn join(&self, other: DefPath) -> DefPath {
        Self(format!("{}::{}", self, other))
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split("::")
    }

    pub fn is_valid_path(path: &str) -> bool {
        Self::validate_path(path, 1)
    }

    pub fn is_valid_qualified_path(path: &str) -> bool {
        Self::validate_path(path, 2)
    }

    /// Helper that validates a path and ensures minimum segment count
    fn validate_path(path: &str, min_segments: usize) -> bool {
        if path.is_empty() {
            return false;
        }

        let segments: Vec<&str> = path.split("::").collect();
        if segments.len() < min_segments {
            return false;
        }

        segments.into_iter().all(|s| is_valid_segment(s))
    }
}

impl Display for DefPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<DefPathSegment> for DefPath {
    fn from(value: DefPathSegment) -> Self {
        Self(value.0)
    }
}

impl TryFrom<&str> for DefPath {
    type Error = ();

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

impl TryFrom<String> for DefPath {
    type Error = ();

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::new(&value).ok_or(())
    }
}

impl FromStr for DefPath {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(s).ok_or(())
    }
}

impl Borrow<str> for DefPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Serialize for DefPath {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DefPath {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(&s).ok_or(()).map_err(|_| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Str(&s), &"a valid path")
        })
    }
}

/// Checks if a segment is valid.
/// Segments must contain only lowercase letters, numbers, and underscores.
/// They may not start or end with an underscore or start with a number.
pub fn is_valid_segment(segment: &str) -> bool {
    if segment.is_empty() || segment.starts_with('_') || segment.ends_with('_') {
        return false;
    }

    let first_char = match segment.chars().next() {
        Some(c) => c,
        None => return false,
    };

    if first_char.is_ascii_digit() {
        return false;
    }

    segment
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
