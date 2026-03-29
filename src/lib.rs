#![feature(duration_millis_float)]

use bevy::state::state::States;

pub mod combat;
pub mod debug;
pub mod graphics;
pub mod input;
pub mod math;
pub mod modding;
pub mod physics;
pub mod player;
pub mod rand;
pub mod serialization;
pub mod world;
pub mod world_gen;

#[derive(States, Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    Boot,
    ModLoading,
    MainMenu,
    InGame,
    Shutdown,
}
