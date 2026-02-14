use std::collections::HashMap;
use std::path::PathBuf;

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

pub use types::{Id, Path, PathSegment, Registry};

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

#[derive(Debug, Default, Resource, Clone)]
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

    pub fn get_by_segment(&self, segment: &PathSegment) -> Option<&ModInfo> {
        self.lookup(segment).and_then(|id| self.get(id))
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

#[derive(Debug, Clone)]
pub struct ModInfo {
    pub path: PathBuf,
    pub metadata: ModMetadata,
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
