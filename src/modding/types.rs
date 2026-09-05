//! Register definitions (defs)

use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    num::NonZeroU32,
    path::PathBuf,
    str::FromStr,
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::modding::DefinitionLoadError;

pub trait Definition: Sized + Send + Sync + 'static {
    const DIR: &'static str;

    fn load(
        mod_id: DefId,
        path: PathBuf,
    ) -> impl Future<Output = Result<(DefId, Self), DefinitionLoadError>> + Send;

    /// Extra setup
    #[allow(unused)]
    fn build(app: &mut App) {}
}

#[derive(Resource, Clone)]
pub struct Registry<T> {
    definitions: Vec<T>,
    ids: Vec<DefId>,
    lookup: HashMap<DefId, DefHandle<T>>,
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
            ids: Vec::new(),
            lookup: HashMap::new(),
        }
    }

    /// Registers a definition with the given id and returns its ID.
    /// If the definition already exists, it is replaced and the existing ID is returned.
    pub fn register(&mut self, id: DefId, def: T) -> DefHandle<T> {
        if let Some(handle) = self.lookup.get(&id).copied() {
            self.definitions[handle.to_index()] = def;
            return handle;
        }

        let handle = DefHandle::from_index(self.definitions.len());
        self.definitions.push(def);
        self.ids.push(id.clone());
        self.lookup.insert(id, handle);

        handle
    }

    pub fn clear(&mut self) {
        self.definitions.clear();
        self.ids.clear();
        self.lookup.clear();
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    /// Retrieves the handle of the definition associated with the given ID.
    pub fn get_handle(&self, id: &str) -> Option<DefHandle<T>> {
        self.lookup.get(id).copied()
    }

    /// Retrieves the ID of the definition associated with the given handle.
    pub fn resolve(&self, handle: DefHandle<T>) -> Option<&DefId> {
        self.ids.get(handle.to_index())
    }

    /// Retrieves the definition associated with the given ID.
    pub fn get(&self, handle: DefHandle<T>) -> Option<&T> {
        self.definitions.get(handle.to_index())
    }

    /// Retrieves the definition associated with the given id.
    pub fn get_by_id(&self, id: &str) -> Option<&T> {
        self.get_handle(id).and_then(|id| self.get(id))
    }

    pub fn contains(&self, handle: DefHandle<T>) -> bool {
        self.definitions.len() > handle.to_index()
    }

    pub fn contains_id(&self, id: &str) -> bool {
        self.lookup.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.definitions.iter()
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (&DefId, &T)> {
        self.ids.iter().zip(self.definitions.iter())
    }

    /// Order is guaranteed to be from lowest to highest id.
    pub fn iter_with_handle(&self) -> impl Iterator<Item = (DefHandle<T>, &T)> {
        self.definitions
            .iter()
            .enumerate()
            .map(|(i, t)| (DefHandle::from_index(i), t))
    }

    pub fn iter_with_id_handle(&self) -> impl Iterator<Item = (DefHandle<T>, &DefId, &T)> {
        self.ids
            .iter()
            .enumerate()
            .zip(self.definitions.iter())
            .map(|((i, id), t)| (DefHandle::from_index(i), id, t))
    }
}

impl<T: Debug> Debug for Registry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (handle, id, definition) in self.iter_with_id_handle() {
            writeln!(f, "{} {}: {:?}", handle.get(), id, definition)?;
        }
        Ok(())
    }
}

impl<T: Display> Display for Registry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (handle, id, definition) in self.iter_with_id_handle() {
            writeln!(f, "{} {}: {}", handle.get(), id, definition)?;
        }
        Ok(())
    }
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            definitions: Vec::new(),
            ids: Vec::new(),
            lookup: HashMap::new(),
        }
    }
}

/// A wrapper over `NonZeroU32` index into a registry.
pub struct DefHandle<T>(NonZeroU32, PhantomData<fn() -> T>);

impl<T> DefHandle<T> {
    pub const fn new(handle: u32) -> Self {
        Self(NonZeroU32::new(handle).unwrap(), PhantomData)
    }

    pub const fn from_index(index: usize) -> Self {
        Self(NonZeroU32::new(index as u32 + 1).unwrap(), PhantomData)
    }

    pub const fn get(&self) -> u32 {
        self.0.get()
    }

    pub const fn to_index(&self) -> usize {
        self.0.get() as usize - 1
    }
}

impl<T> Debug for DefHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle({})", self.0)
    }
}

impl<T> Display for DefHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> Clone for DefHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for DefHandle<T> {}

impl<T> PartialEq for DefHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for DefHandle<T> {}

impl<T> PartialOrd for DefHandle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for DefHandle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> Hash for DefHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// A newtype wrapper over a `String` that ensures the id is valid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref)]
pub struct DefId(String);

impl DefId {
    pub fn new(id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        if !Self::is_valid_id(&id) {
            return None;
        }
        Some(Self(id.into()))
    }

    pub fn new_qualified(id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        if !Self::is_valid_qualified_id(&id) {
            return None;
        }
        Some(Self(id.into()))
    }

    pub fn join(&self, other: DefId) -> DefId {
        Self(format!("{}::{}", self, other))
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split("::")
    }

    pub fn is_valid_id(id: &str) -> bool {
        Self::validate_id(id, 1)
    }

    pub fn is_valid_qualified_id(id: &str) -> bool {
        Self::validate_id(id, 2)
    }

    /// Helper that validates a id and ensures minimum segment count
    fn validate_id(id: &str, min_segments: usize) -> bool {
        if id.is_empty() {
            return false;
        }

        let segments: Vec<&str> = id.split("::").collect();
        if segments.len() < min_segments {
            return false;
        }

        segments.into_iter().all(|s| Self::is_valid_segment(s))
    }

    /// Checks if a segment is valid.
    /// Segments must contain only lowercase letters, numbers, and underscores.
    /// They may not start or end with an underscore or start with a number.
    fn is_valid_segment(segment: &str) -> bool {
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
}

impl Display for DefId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for DefId {
    type Error = ();

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Self::new(value).ok_or(())
    }
}

impl TryFrom<String> for DefId {
    type Error = ();

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::new(&value).ok_or(())
    }
}

impl FromStr for DefId {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::new(s).ok_or(())
    }
}

impl Borrow<str> for DefId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Serialize for DefId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DefId {
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
