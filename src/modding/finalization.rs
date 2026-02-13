use bevy::prelude::*;

use crate::AppState;

pub fn finalize(mut state: ResMut<NextState<AppState>>) {
    info!("Mod loading complete");
    state.set(AppState::MainMenu);
}
