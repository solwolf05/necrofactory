use bevy::prelude::*;

use crate::modding::{ModLoadState, ModRegistry};

pub fn validate_mods(
    mut next_state: ResMut<NextState<ModLoadState>>,
    mut mods: ResMut<ModRegistry>,
) {
    mods.load_order = mods.mods.keys().cloned().collect();

    info!("Mods validation complete");

    next_state.set(ModLoadState::Register);
}
