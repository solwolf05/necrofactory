use std::collections::HashSet;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    modding::{
        ModLoadState,
        types::{Id, Registry},
    },
    world::tile::TileDef,
};

#[derive(Debug, Default, Resource)]
pub struct TileHandles {
    pub(self) missing: Handle<Image>,
    pub pending: HashMap<Id<TileDef>, Handle<Image>>,
    pub complete: HashMap<Id<TileDef>, Handle<Image>>,
}

pub fn begin_asset_loading(
    tiles: Res<Registry<TileDef>>,
    mut handles: ResMut<TileHandles>,
    asset_server: Res<AssetServer>,
) {
    let missing = asset_server.load("missing.png");
    handles.missing = missing.clone();
    for (id, _, tile) in tiles.iter_with_id() {
        let handle = asset_server.load(&tile.sprite_path);
        handles.pending.insert(id, handle);
    }
}

pub fn check_assets_loaded(
    mut next_state: ResMut<NextState<ModLoadState>>,
    mut handles: ResMut<TileHandles>,
    registry: Res<Registry<TileDef>>,
    asset_server: Res<AssetServer>,
) {
    let mut to_complete = Vec::new();
    let mut to_fail = Vec::new();

    for (&id, handle) in handles.pending.iter() {
        let load_state = asset_server.load_state(handle);
        match load_state {
            bevy::asset::LoadState::NotLoaded | bevy::asset::LoadState::Loading => continue,
            bevy::asset::LoadState::Loaded => {
                to_complete.push(id);
            }
            bevy::asset::LoadState::Failed(asset_load_error) => {
                to_fail.push((id, asset_load_error));
            }
        }
    }

    for id in to_complete {
        if let Some((id, handle)) = handles.pending.remove_entry(&id) {
            handles.complete.insert(id, handle);
        }
    }

    for (id, asset_load_error) in to_fail {
        handles.pending.remove_entry(&id);
        let missing = handles.missing.clone();
        handles.complete.insert(id, missing);
        error!(
            "Failed to load asset for tile {}: {}",
            registry.resolve(id).unwrap(),
            asset_load_error
        );
    }

    if handles.pending.is_empty() {
        info!("Asset loading complete");
        next_state.set(ModLoadState::Finalize);
    }
}
