use std::fs;

use bevy::prelude::*;

use crate::modding::{ModInfo, ModLoadState, ModMetadata, ModRegistry, mods_path};

pub fn discover_mods(
    mut next_state: ResMut<NextState<ModLoadState>>,
    mut mods: ResMut<ModRegistry>,
) {
    let entries = match fs::read_dir(mods_path()) {
        Ok(e) => e.flatten(),
        Err(e) => {
            error!("Error reading mods dir: {}", e);
            return;
        }
    };

    for dir in entries {
        let path = dir.path();
        let metadata_path = path.join("mod.toml");

        // Metadata
        let Ok(metadata_str) = fs::read_to_string(&metadata_path) else {
            continue;
        };
        let metadata: ModMetadata = match toml::from_str(&metadata_str) {
            Ok(m) => m,
            Err(e) => {
                error!("Error parsing {}: {}", metadata_path.display(), e);
                continue;
            }
        };

        let enabled = !path.join("disabled").exists();
        let mod_info = ModInfo {
            path,
            metadata,
            enabled,
        };

        mods.register(mod_info.metadata.id.clone(), mod_info);
    }
    info!("Mod discovery complete");

    next_state.set(ModLoadState::Validate);
}
