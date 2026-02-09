use std::{collections::HashMap, path::PathBuf};

use bevy::{
    asset::io::{AssetSourceBuilder, AssetSourceId, file::FileAssetReader},
    prelude::*,
};
use bevy_modding::prelude::*;
use bevy_modding_input::InputAction;

use loader::{discover_mods, load_inputs, load_tiles};
use serialisation::Metadata;

use loader::load_tile_textures;

pub use loader::{TileTextures, all_tile_textures_loaded};

use crate::world::tile::TileDef;

mod loader;
mod serialisation;

/// Loads mods at the start of the game and registers their types in the registry.
pub struct ModLoadPlugin;

impl Plugin for ModLoadPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Mods>()
            .init_resource::<Registry<TileDef>>()
            .init_resource::<TileTextures>()
            .add_systems(PreModLoad, discover_mods)
            .add_systems(ModLoad, (load_inputs, load_tiles))
            .add_systems(PostModLoad, (load_tile_textures, check_registries));

        app.register_asset_source(
            AssetSourceId::Name("mods".into()),
            AssetSourceBuilder::new(|| Box::new(FileAssetReader::new(mods_path()))),
        );
    }
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

#[derive(Debug, Default, Resource)]
pub struct Mods {
    pub mods: HashMap<String, Mod>,
}

#[derive(Debug)]
pub struct Mod {
    pub metadata: Metadata,
    pub path: PathBuf,
}

impl Mod {
    pub fn new(metadata: Metadata, path: PathBuf) -> Self {
        Self { metadata, path }
    }

    pub fn join_path(&self, path: RegistryPath) -> Option<RegistryPath> {
        RegistryPath::new(&format!("{}{}", self.metadata.id, path))
    }
}
