use std::fmt::{Debug, Display};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};

use bevy::{
    asset::io::{AssetSourceBuilder, AssetSourceId, file::FileAssetReader},
    prelude::*,
};

use serde::Deserialize;

use crate::{
    AppState,
    input::InputAction,
    modding::{
        asset_loading::{begin_asset_loading, check_assets_loaded},
        discovery::discover_mods,
        finalization::finalize,
        registration::{
            Active, Complete, Pending, check_registries_loaded, discover_definitions,
            log_registration, poll_registration, spawn_registration,
        },
        validation::validate_mods,
    },
    world::tile::TileDef,
};

pub use asset_loading::TileHandles;

mod asset_loading;
mod discovery;
mod finalization;
mod registration;
mod types;
mod validation;

pub use types::{Id, PathSegment, Registry};

/// Loads mods at the start of the game and registers their types in the registry.
pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_sub_state::<ModLoadState>()
            .init_resource::<ModRegistry>()
            .init_resource::<Pending>()
            .init_resource::<Active>()
            .init_resource::<Complete>()
            .init_resource::<TileHandles>()
            .init_resource::<Registry<InputAction>>()
            .init_resource::<Registry<TileDef>>()
            .add_systems(OnEnter(ModLoadState::Discover), discover_mods)
            .add_systems(OnEnter(ModLoadState::Validate), validate_mods)
            .add_systems(OnExit(ModLoadState::Validate), check_mods)
            .add_systems(OnEnter(ModLoadState::Register), discover_definitions)
            .add_systems(
                Update,
                (
                    spawn_registration,
                    poll_registration,
                    log_registration,
                    check_registries_loaded.after(poll_registration),
                )
                    .run_if(in_state(ModLoadState::Register)),
            )
            .add_systems(OnEnter(ModLoadState::LoadAssets), begin_asset_loading)
            .add_systems(
                Update,
                check_assets_loaded.run_if(in_state(ModLoadState::LoadAssets)),
            )
            .add_systems(OnEnter(ModLoadState::Finalize), finalize)
            .add_systems(OnEnter(ModLoadState::Finalize), check_registries);
    }
}

pub struct ModAssetSourcePlugin;

impl Plugin for ModAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            AssetSourceId::Name("mods".into()),
            AssetSourceBuilder::new(|| Box::new(FileAssetReader::new(mods_path()))),
        );
    }
}

#[derive(SubStates, Debug, Default, Clone, Eq, PartialEq, Hash)]
#[source(AppState = AppState::ModLoading)]
pub enum ModLoadState {
    #[default]
    Discover,
    Validate,
    Register,
    LoadAssets,
    Finalize,
}

fn mods_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mods");
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("mods")
}

fn check_registries(inputs: Res<Registry<InputAction>>, tiles: Res<Registry<TileDef>>) {
    info!("Inputs:\n{:?}", *inputs);
    info!("Tiles:\n{:?}", *tiles);
}

fn check_mods(mods: Res<ModRegistry>) {
    info!("Mods:\n{}", *mods);
    info!(
        "Mod load order: {}",
        mods.load_order
            .iter()
            .map(|&id| mods.resolve(id).unwrap().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[derive(Default, Resource, Clone)]
pub struct ModRegistry {
    mods: Vec<(PathSegment, ModInfo)>,
    lookup: HashMap<PathSegment, Id<ModInfo>>,
    pub load_order: Vec<Id<ModInfo>>,
}

impl ModRegistry {
    pub fn register(&mut self, segment: PathSegment, mod_info: ModInfo) -> Id<ModInfo> {
        if let Some(id) = self.lookup.get(&segment).copied() {
            self.mods[id.get() as usize].1 = mod_info;
            return id;
        }

        let id = Id::new(self.mods.len() as u32);
        self.mods.push((segment.clone(), mod_info));
        self.lookup.insert(segment, id);

        id
    }

    pub fn enable(&mut self, id: Id<ModInfo>) {
        if let Some(mod_info) = self.get_mut(id) {
            mod_info.enable();
        }
    }

    pub fn enable_segment(&mut self, segment: &PathSegment) {
        if let Some(mod_info) = self.get_by_segment_mut(segment) {
            mod_info.enable();
        }
    }

    pub fn disable(&mut self, id: Id<ModInfo>) {
        if let Some(mod_info) = self.get_mut(id) {
            mod_info.disable();
        }
    }

    pub fn disable_segment(&mut self, segment: &PathSegment) {
        if let Some(mod_info) = self.get_by_segment_mut(segment) {
            mod_info.disable();
        }
    }

    pub fn len(&self) -> usize {
        self.mods.len()
    }

    pub fn lookup(&self, segment: &PathSegment) -> Option<Id<ModInfo>> {
        self.lookup.get(&segment).copied()
    }

    pub fn resolve(&self, id: Id<ModInfo>) -> Option<&PathSegment> {
        self.mods.get(id.get() as usize).map(|r| &r.0)
    }

    pub fn get(&self, id: Id<ModInfo>) -> Option<&ModInfo> {
        self.mods.get(id.get() as usize).map(|r| &r.1)
    }

    fn get_mut(&mut self, id: Id<ModInfo>) -> Option<&mut ModInfo> {
        self.mods.get_mut(id.get() as usize).map(|r| &mut r.1)
    }

    pub fn get_by_segment(&self, segment: &PathSegment) -> Option<&ModInfo> {
        self.lookup(segment).and_then(|id| self.get(id))
    }

    pub fn get_by_segment_mut(&mut self, segment: &PathSegment) -> Option<&mut ModInfo> {
        self.lookup(segment).and_then(|id| self.get_mut(id))
    }

    pub fn contains(&self, id: Id<ModInfo>) -> bool {
        self.mods.len() > id.get() as usize
    }

    pub fn contains_path(&self, segment: &PathSegment) -> bool {
        self.lookup.contains_key(segment)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PathSegment, &ModInfo)> {
        self.mods.iter().map(|(s, t)| (s, t))
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (Id<ModInfo>, &PathSegment, &ModInfo)> {
        self.mods
            .iter()
            .enumerate()
            .map(|(i, (s, t))| (Id::new(i as u32), s, t))
    }
}

impl Debug for ModRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, segment, mod_info) in self.iter_with_id() {
            writeln!(f, "{} {}: {:?}", id.get(), segment, mod_info)?;
        }
        Ok(())
    }
}

impl Display for ModRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, segment, mod_info) in self.iter_with_id() {
            writeln!(f, "{} {}: {}", id.get(), segment, mod_info)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModInfo {
    path: PathBuf,
    metadata: ModMetadata,
    enabled: bool,
}

impl ModInfo {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn id(&self) -> &PathSegment {
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

    pub fn dependencies(&self) -> &HashMap<PathSegment, String> {
        &self.metadata.dependencies
    }

    pub fn optional_dependencies(&self) -> &HashMap<PathSegment, String> {
        &self.metadata.optional_dependencies
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        let _ = fs::remove_file(self.disable_path());
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let _ = fs::write(self.disable_path(), []);
        self.enabled = false;
    }

    pub fn disable_path(&self) -> PathBuf {
        self.path.join("disabled")
    }
}

impl Display for ModInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} by {}; dependencies: {}; optional dependencies: {}; path: {}; {}",
            self.name(),
            self.version(),
            self.author(),
            self.dependencies()
                .into_iter()
                .map(|(i, v)| format!("{} {}", i, v))
                .collect::<Vec<_>>()
                .join(", "),
            self.optional_dependencies()
                .into_iter()
                .map(|(i, v)| format!("{} {}", i, v))
                .collect::<Vec<_>>()
                .join(", "),
            self.path().display(),
            if self.enabled() {
                "enabled"
            } else {
                "disabled"
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct ModMetadata {
    pub id: PathSegment,
    pub name: String,
    pub version: String,
    pub author: String,
    pub dependencies: HashMap<PathSegment, String>,
    pub optional_dependencies: HashMap<PathSegment, String>,
}

impl<'de> Deserialize<'de> for ModMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMetadata {
            pub id: PathSegment,
            pub name: String,
            pub version: String,
            pub author: String,
            pub dependencies: Option<HashMap<PathSegment, String>>,
            pub optional_dependencies: Option<HashMap<PathSegment, String>>,
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
