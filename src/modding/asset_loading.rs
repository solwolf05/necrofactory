use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    modding::{
        ModLoadState,
        types::{Id, Registry},
    },
    world::tile::TileDef,
};

#[derive(Debug, Default, Resource)]
pub struct PendingSprites(pub HashMap<Id<TileDef>, Handle<Image>>);

#[derive(Debug, Default, Resource)]
pub struct TileSprites {
    missing: Handle<Image>,
    sprites: HashMap<Id<TileDef>, Handle<Image>>,
}

impl TileSprites {
    pub fn get(&self, id: Id<TileDef>) -> Handle<Image> {
        self.sprites.get(&id).unwrap_or(&self.missing).clone()
    }
}

pub fn begin_asset_loading(
    mut commands: Commands,
    tiles: Res<Registry<TileDef>>,
    mut sprites: ResMut<TileSprites>,
    asset_server: Res<AssetServer>,
) {
    let missing = asset_server.load("missing.png");
    sprites.missing = missing.clone();

    sprites.sprites.clear();

    let mut pending = PendingSprites::default();
    for (id, _, tile) in tiles.iter_with_id() {
        let handle = asset_server.load(&tile.sprite_path);
        pending.0.insert(id, handle);
    }
    commands.insert_resource(pending);
}

pub fn check_loaded(
    mut next_state: ResMut<NextState<ModLoadState>>,
    mut sprites: ResMut<TileSprites>,
    mut pending: ResMut<PendingSprites>,
    registry: Res<Registry<TileDef>>,
    asset_server: Res<AssetServer>,
) {
    let mut to_complete = Vec::new();
    let mut to_fail = Vec::new();

    for (&id, handle) in pending.0.iter() {
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
        if let Some((id, handle)) = pending.0.remove_entry(&id) {
            sprites.sprites.insert(id, handle);
        }
    }

    for (id, asset_load_error) in to_fail {
        pending.0.remove_entry(&id);
        error!(
            "Failed to load asset for tile {}: {}",
            registry.resolve(id).unwrap(),
            asset_load_error
        );
    }

    if pending.0.is_empty() {
        info!("Asset loading complete");
        next_state.set(ModLoadState::Finalize);
    }
}

pub fn cleanup(mut commands: Commands) {
    commands.remove_resource::<PendingSprites>();
}
