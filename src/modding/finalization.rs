use bevy::prelude::*;

use crate::GameState;

pub fn finalize(mut state: ResMut<NextState<GameState>>) {
    info!("Mod loading complete");
    state.set(GameState::InGame);
}
