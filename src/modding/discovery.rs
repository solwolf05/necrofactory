use std::{fs, time::Instant};

use bevy::prelude::*;

use crate::Config;
use crate::modding::{ModInfo, ModLoadState, ModMetadata, ModRegistry, mods_path};

pub fn discover_mods(
    mut next_state: ResMut<NextState<ModLoadState>>,
    mut mods: ResMut<ModRegistry>,
    mut config: ResMut<Config>,
) {
    let instant = Instant::now();

    let entries = match fs::read_dir(mods_path()) {
        Ok(e) => e.flatten(),
        Err(e) => {
            error!("Error reading mods dir: {}", e);
            next_state.set(ModLoadState::Validate);
            return;
        }
    };

    // mods.clear();

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

        let mod_info = ModInfo { path, metadata };
        let id = mod_info.id().clone();

        mods.register(id.clone(), mod_info);

        if config.installed_mods.contains(&id) {
            mods.disable(&id);
        } else if !config.enabled_mods.contains(&id) {
            config.enabled_mods.push(id);
        }
    }

    let elapsed = instant.elapsed();
    info!("Mod discovery complete ({}ms)", elapsed.as_millis_f32());

    fs::write(
        "/home/solwolf/dev/necrofactory/config.toml",
        toml::to_string_pretty(config.into_inner()).unwrap(),
    )
    .unwrap();

    next_state.set(ModLoadState::Validate);
}
