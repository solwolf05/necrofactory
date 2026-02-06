use std::fs;

use bevy::prelude::*;
use bevy_modding::prelude::*;
use bevy_modding_input::{InputAction, InputBinding};
use serde::Deserialize;

use crate::modding::{Mod, Mods, mods_path, serialisation::Metadata};

pub fn discover_mods(mut mods: ResMut<Mods>) {
    let entries = match fs::read_dir(mods_path()) {
        Ok(e) => e.flatten(),
        Err(e) => {
            error!("error reading mods dir: {}", e);
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
                error!("error parsing {}: {}", metadata_path.display(), e);
                error += 1;
                continue;
            }
        };

        if !RegPath::is_valid_segment(&metadata.id) {
            error!("invalid mod id: {}", metadata.id);
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
                        "error while parsing input definition at {}: {}",
                        path.display(),
                        e
                    );
                    continue;
                }
            };

            let reg_path = match RegPath::from_parts(mod_id.as_ref(), raw_input.id.as_ref()) {
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

#[derive(Deserialize)]
struct RawInputAction {
    id: String,
    name: String,
    default: InputBinding,
}
