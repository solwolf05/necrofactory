use bevy::prelude::*;

use crate::modding::ModLoadState;

pub fn load_assets(mut state: ResMut<NextState<ModLoadState>>) {
    info!("Asset loading complete");
    state.set(ModLoadState::Finalize);
}
