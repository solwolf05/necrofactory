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
        asset_loading::load_assets,
        discovery::discover_mods,
        finalization::finalize,
        registration::{
            Active, Complete, Pending, check_registries_loaded, discover_definitions,
            log_registration, poll_registration, spawn_registration,
        },
        types::{PathSegment, Registry},
        validation::validate_mods,
    },
    world::tile::TileDef,
};

mod asset_loading;
mod discovery;
mod finalization;
mod registration;
pub mod types;
mod validation;

/// Loads mods at the start of the game and registers their types in the registry.
pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_sub_state::<ModLoadState>()
            .init_resource::<ModRegistry>()
            .init_resource::<Pending>()
            .init_resource::<Active>()
            .init_resource::<Complete>()
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
            .add_systems(OnEnter(ModLoadState::LoadAssets), load_assets)
            .add_systems(OnEnter(ModLoadState::Finalize), finalize)
            .add_systems(OnEnter(ModLoadState::Finalize), check_registries);

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
    pub mods: HashMap<PathSegment, ModInfo>,
    pub load_order: Vec<PathSegment>,
}

impl ModRegistry {
    pub fn iter(&self) -> impl Iterator<Item = &ModInfo> {
        self.load_order.iter().map(|id| self.mods.get(id).unwrap())
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
