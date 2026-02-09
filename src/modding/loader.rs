use std::fs;

use bevy::prelude::*;
use bevy_modding::prelude::*;
use bevy_modding_input::{InputAction, InputBinding};
use serde::Deserialize;

use crate::{
    modding::{Mod, Mods, mods_path, serialisation::Metadata},
    world::tile::TileDef,
};

#[derive(Debug, Default, Resource)]
pub struct TileTextures {
    pub handles: Vec<(Id<TileDef>, Handle<Image>)>,
}

pub fn all_tile_textures_loaded(server: Res<AssetServer>, pending: Res<TileTextures>) {
    pending
        .handles
        .iter()
        .all(|(_, h)| server.wait(h));
}

pub fn discover_mods(mut mods: ResMut<Mods>) {
    let entries = match fs::read_dir(mods_path()) {
        Ok(e) => e.flatten(),
        Err(e) => {
            error!("Error reading mods dir: {}", e);
            return;
        }
    };

    let mut success = 0;
    let mut error = 0;
    let mut skipped = 0;

    for dir in entries {
        let path = dir.path();
        if path
            .file_name()
            .and_then(|dir| dir.to_str())
            .map(|dir| dir.starts_with("_"))
            .unwrap_or_default()
        {
            // Skip mod dirs that start with "_"
            // This is for easier enabling and disabling of mods
            skipped += 1;
            continue;
        }
        let metadata_path = path.join("mod.toml");

        // Metadata
        let Ok(bytes) = fs::read(&metadata_path) else {
            continue;
        };
        let metadata: Metadata = match toml::from_slice(&bytes) {
            Ok(m) => m,
            Err(e) => {
                error!("Error parsing {}: {}", metadata_path.display(), e);
                error += 1;
                continue;
            }
        };

        if !RegistryPath::is_valid_segment(&metadata.id) {
            error!("Invalid mod id: {}", metadata.id);
            error += 1;
            continue;
        };

        let mod_data = Mod { metadata, path };
        mods.mods.insert(mod_data.metadata.id.clone(), mod_data);
        success += 1;
    }
    info!(
        "Mods discovered: success: {}, error: {}, skipped: {}",
        success, error, skipped
    );
}

pub fn load_inputs(mods: Res<Mods>, mut registry: ResMut<Registry<InputAction>>) {
    for (mod_id, mod_data) in &mods.mods {
        let path = mod_data.path.join("inputs");
        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(string) = fs::read_to_string(&path) else {
                continue;
            };
            let raw_input: RawInputAction = match ron::from_str(&string) {
                Ok(i) => i,
                Err(e) => {
                    error!(
                        "Error while parsing input definition at {}: {}",
                        path.display(),
                        e
                    );
                    continue;
                }
            };

            let reg_path = match RegistryPath::from_parts(mod_id, &raw_input.id) {
                Some(p) => p,
                None => {
                    error!("Invalid registry path: {}::{}", mod_id, raw_input.id);
                    continue;
                }
            };

            let input = InputAction {
                name: raw_input.name,
                default: raw_input.default,
            };

            registry.register(reg_path, input);
        }
    }
    info!("Loaded mod inputs")
}

pub fn load_tiles(mods: Res<Mods>, mut registry: ResMut<Registry<TileDef>>) {
    registry.register(
        "core::none",
        TileDef {
            sprite_path: "missing.png".to_owned(),
        },
    );
    registry.register(
        "core::some",
        TileDef {
            sprite_path: "block.png".to_owned(),
        },
    );

    for (mod_id, mod_data) in &mods.mods {
        let path = mod_data.path.join("tiles");
        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(string) = fs::read_to_string(&path) else {
                continue;
            };
            let raw_tile: RawTile = match ron::from_str(&string) {
                Ok(i) => i,
                Err(e) => {
                    error!(
                        "Error while parsing tile definition at {}: {}",
                        path.display(),
                        e
                    );
                    continue;
                }
            };

            let reg_path = match RegistryPath::from_parts(mod_id, &raw_tile.id) {
                Some(p) => p,
                None => {
                    error!("Invalid registry path: {}::{}", mod_id, raw_tile.id);
                    continue;
                }
            };

            let tile = TileDef {
                sprite_path: raw_tile.sprite_path,
            };

            registry.register(reg_path, tile);
        }
    }
    info!("Loaded mod tiles")
}

pub fn load_tile_textures(
    tiles: Res<Registry<TileDef>>,
    mut textures: ResMut<TileTextures>,
    asset_server: Res<AssetServer>,
) {
    for (id, _, tile) in tiles.iter_with_id() {
        let handle = asset_server.load(&tile.sprite_path);
        textures.handles.push((id, handle));
    }
}

#[derive(Deserialize)]
struct RawTile {
    id: String,
    sprite_path: String,
}

#[derive(Deserialize)]
struct RawInputAction {
    id: String,
    name: String,
    default: InputBinding,
}
