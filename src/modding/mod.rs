use std::{collections::HashMap, path::PathBuf};

use bevy::{
    asset::io::{AssetSourceBuilder, AssetSourceId, file::FileAssetReader},
    prelude::*,
};
use bevy_modding::prelude::*;
use bevy_modding_input::InputAction;

use crate::modding::{
    loader::{discover_mods, load_inputs},
    serialisation::Metadata,
};

mod loader;
mod serialisation;

/// Loads mods at the start of the game and registers their types in the registry.
pub struct ModLoadPlugin;

impl Plugin for ModLoadPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_resource::<Mods>()
            .add_systems(PreModLoad, discover_mods)
            .add_systems(ModLoad, load_inputs)
            .add_systems(PostModLoad, check_registries);

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

fn check_registries(input: Res<Registry<InputAction>>) {
    info!("Inputs:\n{:?}", *input);
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

    pub fn join_path(&self, path: RegPath) -> Option<RegPath> {
        RegPath::new(&format!("{}{}", self.metadata.id, path))
    }
}
