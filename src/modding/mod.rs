use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use bevy::{
    asset::io::{AssetSourceBuilder, AssetSourceId, file::FileAssetReader},
    prelude::*,
};

use serde::Deserialize;

use crate::GameState;

use asset_loading::begin_asset_loading;
use discovery::discover_mods;
use finalization::finalize;
use validation::validate_mods;

pub use asset_loading::TileSprites;
pub use registration::DefinitionLoadError;
pub use resolution::ResolvedRegistry;
pub use types::*;

mod asset_loading;
mod discovery;
mod finalization;
mod registration;
mod resolution;
mod types;
mod validation;

/// Loads mods at the start of the game and registers their types in the registry.
pub struct ModdingPlugin;

impl Plugin for ModdingPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_sub_state::<ModLoadState>()
            .init_resource::<ModRegistry>()
            .init_resource::<TileSprites>();

        app.add_systems(OnEnter(ModLoadState::Discover), discover_mods);

        app.add_systems(OnEnter(ModLoadState::Validate), validate_mods)
            .add_systems(OnExit(ModLoadState::Validate), check_mods);

        app.add_systems(OnEnter(ModLoadState::Register), registration::setup)
            .add_systems(
                Update,
                (registration::log, registration::check_loaded)
                    .run_if(in_state(ModLoadState::Register)),
            )
            .add_systems(OnExit(ModLoadState::Register), registration::cleanup);

        app.add_systems(
            Update,
            resolution::cleanup.run_if(in_state(ModLoadState::Resolve)),
        );

        app.add_systems(OnEnter(ModLoadState::LoadAssets), begin_asset_loading)
            .add_systems(
                Update,
                asset_loading::check_loaded.run_if(in_state(ModLoadState::LoadAssets)),
            )
            .add_systems(OnExit(ModLoadState::LoadAssets), asset_loading::cleanup);

        app.add_systems(OnEnter(ModLoadState::Finalize), finalize);
    }
}

pub struct ModAssetSourcePlugin;

impl Plugin for ModAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            AssetSourceId::Name("mods".into()),
            AssetSourceBuilder::new(|| Box::new(FileAssetReader::new(mods_dir()))),
        );
    }
}

pub struct DefinitionPlugin<D: Definition + Debug>(PhantomData<D>);

impl<D: Definition + Debug> Plugin for DefinitionPlugin<D> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Registry<D>>()
            .add_systems(
                OnEnter(ModLoadState::Register),
                (
                    registration::discover::<D>.after(registration::setup),
                    registration::clear::<D>,
                ),
            )
            .add_systems(
                Update,
                (registration::spawn::<D>, registration::poll::<D>)
                    .run_if(in_state(ModLoadState::Register)),
            );
        #[cfg(debug_assertions)]
        app.add_systems(OnExit(ModLoadState::Register), registration::check::<D>);

        D::build(app);
    }
}

impl<D: Definition + Debug> Default for DefinitionPlugin<D> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(SubStates, Debug, Default, Clone, Eq, PartialEq, Hash)]
#[source(GameState = GameState::ModLoading)]
pub enum ModLoadState {
    /// Discover and load mod metadata from mods folder.
    #[default]
    Discover,
    /// Validate dependencies and determine mod load order.
    Validate,
    /// Register definitions into registries.
    Register,
    /// Resolve ids to handles in definitions.
    Resolve,
    /// Load assets and create spritemaps.
    LoadAssets,
    /// Finalize mod loading and check registries.
    Finalize,
}

fn mods_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mods");
        if candidate.exists() {
            return candidate;
        }
    }

    #[cfg(debug_assertions)]
    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("mods");

    #[cfg(not(debug_assertions))]
    panic!("unable to find mods path");
}

fn check_mods(mods: Res<ModRegistry>) {
    debug!("Mods:\n{}", *mods);
    debug!(
        "Mod load order: {}",
        mods.load_order
            .iter()
            .map(|&handle| mods.get_id(handle).unwrap().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[derive(Default, Resource, Clone)]
pub struct ModRegistry {
    mods: Vec<ModInfo>,
    ids: Vec<DefId>,
    disabled_lookup: HashMap<DefId, DefHandle<ModInfo>>,
    enabled_lookup: HashMap<DefId, DefHandle<ModInfo>>,
    pub load_order: Vec<DefHandle<ModInfo>>,
}

impl ModRegistry {
    pub fn register(&mut self, id: DefId, mod_info: ModInfo) -> DefHandle<ModInfo> {
        if let Some(handle) = self.enabled_lookup.get(&id).copied() {
            self.mods[handle.to_index()] = mod_info;
            return handle;
        }
        if let Some(handle) = self.disabled_lookup.get(&id).copied() {
            self.mods[handle.to_index()] = mod_info;
            return handle;
        }

        let handle = DefHandle::from_index(self.mods.len());
        self.mods.push(mod_info);
        self.ids.push(id.clone());
        self.enabled_lookup.insert(id, handle);

        handle
    }

    pub fn enable(&mut self, id: &DefId) {
        if let Some(mod_info) = self.disabled_lookup.remove(id) {
            self.enabled_lookup.insert(id.clone(), mod_info);
        }
    }

    pub fn disable(&mut self, id: &DefId) {
        if let Some(mod_info) = self.enabled_lookup.remove(id) {
            self.disabled_lookup.insert(id.clone(), mod_info);
        }
    }

    pub fn clear(&mut self) {
        self.mods.clear();
        self.ids.clear();
        self.disabled_lookup.clear();
        self.enabled_lookup.clear();
        self.load_order.clear();
    }

    pub fn len(&self) -> usize {
        self.mods.len()
    }

    pub fn len_disabled(&self) -> usize {
        self.disabled_lookup.len()
    }

    pub fn len_enabled(&self) -> usize {
        self.enabled_lookup.len()
    }

    pub fn get_handle_disabled(&self, id: &DefId) -> Option<DefHandle<ModInfo>> {
        self.disabled_lookup.get(id).copied()
    }

    pub fn get_handle_enabled(&self, id: &DefId) -> Option<DefHandle<ModInfo>> {
        self.enabled_lookup.get(id).copied()
    }

    pub fn get_id(&self, handle: DefHandle<ModInfo>) -> Option<&DefId> {
        self.ids.get(handle.to_index())
    }

    pub fn get(&self, handle: DefHandle<ModInfo>) -> Option<&ModInfo> {
        self.mods.get(handle.to_index())
    }

    pub fn get_mut(&mut self, handle: DefHandle<ModInfo>) -> Option<&mut ModInfo> {
        self.mods.get_mut(handle.to_index())
    }

    pub fn get_disabled_by_id(&self, id: &DefId) -> Option<&ModInfo> {
        self.get_handle_disabled(id)
            .and_then(|handle| self.get(handle))
    }

    pub fn get_disabled_by_id_mut(&mut self, id: &DefId) -> Option<&mut ModInfo> {
        self.get_handle_disabled(id)
            .and_then(|handle| self.get_mut(handle))
    }

    pub fn get_enabled_by_id(&self, id: &DefId) -> Option<&ModInfo> {
        self.get_handle_enabled(id)
            .and_then(|handle| self.get(handle))
    }

    pub fn get_enabled_by_id_mut(&mut self, id: &DefId) -> Option<&mut ModInfo> {
        self.get_handle_enabled(id)
            .and_then(|handle| self.get_mut(handle))
    }

    pub fn contains(&self, handle: DefHandle<ModInfo>) -> bool {
        self.mods.len() > handle.to_index()
    }

    pub fn disabled_contains_id(&self, id: &DefId) -> bool {
        self.disabled_lookup.contains_key(id)
    }

    pub fn enabled_contains_id(&self, id: &DefId) -> bool {
        self.enabled_lookup.contains_key(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ModInfo> {
        self.mods.iter()
    }

    pub fn iter_disabled(&self) -> impl Iterator<Item = &ModInfo> {
        self.disabled_lookup
            .values()
            .filter_map(|&handle| self.get(handle))
    }

    pub fn iter_enabled(&self) -> impl Iterator<Item = &ModInfo> {
        self.enabled_lookup
            .values()
            .filter_map(|&handle| self.get(handle))
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (&DefId, &ModInfo)> {
        self.ids.iter().zip(self.mods.iter())
    }

    pub fn iter_disabled_with_id(&self) -> impl Iterator<Item = (&DefId, &ModInfo)> {
        self.disabled_lookup
            .values()
            .filter_map(|&handle| self.get_id(handle).zip(self.get(handle)))
    }

    pub fn iter_enabled_with_id(&self) -> impl Iterator<Item = (&DefId, &ModInfo)> {
        self.enabled_lookup
            .values()
            .filter_map(|&handle| self.get_id(handle).zip(self.get(handle)))
    }

    /// Order is guaranteed to be from lowest to highest id.
    pub fn iter_with_handle(&self) -> impl Iterator<Item = (DefHandle<ModInfo>, &ModInfo)> {
        self.mods
            .iter()
            .enumerate()
            .map(|(i, t)| (DefHandle::from_index(i), t))
    }

    pub fn iter_disabled_with_handle(
        &self,
    ) -> impl Iterator<Item = (DefHandle<ModInfo>, &ModInfo)> {
        self.disabled_lookup
            .values()
            .filter_map(|&handle| self.get(handle).map(|m| (handle, m)))
    }

    pub fn iter_enabled_with_handle(&self) -> impl Iterator<Item = (DefHandle<ModInfo>, &ModInfo)> {
        self.enabled_lookup
            .values()
            .filter_map(|&handle| self.get(handle).map(|m| (handle, m)))
    }

    pub fn iter_with_id_handle(
        &self,
    ) -> impl Iterator<Item = (DefHandle<ModInfo>, &DefId, &ModInfo)> {
        self.ids
            .iter()
            .enumerate()
            .zip(self.mods.iter())
            .map(|((i, id), t)| (DefHandle::from_index(i), id, t))
    }

    pub fn iter_disabled_with_id_handle(
        &self,
    ) -> impl Iterator<Item = (DefHandle<ModInfo>, &DefId, &ModInfo)> {
        self.disabled_lookup.values().filter_map(|&handle| {
            self.get_id(handle)
                .zip(self.get(handle))
                .map(|(id, m)| (handle, id, m))
        })
    }

    pub fn iter_enabled_with_id_handle(
        &self,
    ) -> impl Iterator<Item = (DefHandle<ModInfo>, &DefId, &ModInfo)> {
        self.enabled_lookup.values().filter_map(|&handle| {
            self.get_id(handle)
                .zip(self.get(handle))
                .map(|(id, m)| (handle, id, m))
        })
    }
}

impl Debug for ModRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (handle, id, mod_info) in self.iter_with_id_handle() {
            writeln!(f, "{} {}: {:?}", handle.get(), id, mod_info)?;
        }
        Ok(())
    }
}

impl Display for ModRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (handle, id, mod_info) in self.iter_with_id_handle() {
            writeln!(f, "{} {}: {}", handle.get(), id, mod_info)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModInfo {
    path: PathBuf,
    metadata: ModMetadata,
}

impl ModInfo {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn id(&self) -> &DefId {
        &self.metadata.id
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn version(&self) -> &str {
        &self.metadata.version
    }

    pub fn author(&self) -> &str {
        &self.metadata.author
    }

    pub fn dependencies(&self) -> &HashMap<DefId, String> {
        &self.metadata.dependencies
    }

    pub fn optional_dependencies(&self) -> &HashMap<DefId, String> {
        &self.metadata.optional_dependencies
    }
}

impl Display for ModInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} by {}", self.name(), self.version(), self.author())?;
        if self.dependencies().len() != 0 {
            write!(
                f,
                "; dependencies: {}",
                self.dependencies()
                    .into_iter()
                    .map(|(i, v)| format!("{} {}", i, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        if self.optional_dependencies().len() != 0 {
            write!(
                f,
                "; optional dependencies: {}",
                self.optional_dependencies()
                    .into_iter()
                    .map(|(i, v)| format!("{} {}", i, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        write!(
            f,
            "; path: mods/{};",
            self.path().strip_prefix(mods_dir()).unwrap().display(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ModMetadata {
    pub id: DefId,
    pub name: String,
    pub version: String,
    pub author: String,
    pub dependencies: HashMap<DefId, String>,
    pub optional_dependencies: HashMap<DefId, String>,
}

impl<'de> Deserialize<'de> for ModMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMetadata {
            pub id: DefId,
            pub name: String,
            pub version: String,
            pub author: String,
            pub dependencies: Option<HashMap<DefId, String>>,
            pub optional_dependencies: Option<HashMap<DefId, String>>,
        }

        let raw = RawMetadata::deserialize(deserializer)?;
        Ok(ModMetadata {
            id: raw.id,
            name: raw.name,
            version: raw.version,
            author: raw.author,
            dependencies: raw.dependencies.unwrap_or_default(),
            optional_dependencies: raw.optional_dependencies.unwrap_or_default(),
        })
    }
}
