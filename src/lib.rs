#![feature(duration_millis_float)]
#![feature(iter_map_windows)]
#![feature(option_reference_flattening)]

use std::path::PathBuf;

use bevy::{ecs::resource::Resource, state::state::States};
use serde::{Deserialize, Serialize};

use crate::modding::DefId;

pub mod combat;
pub mod debug;
pub mod factory;
pub mod graphics;
pub mod input;
pub mod item;
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

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct Config {
    mods_dir: PathBuf,
    enabled_mods: Vec<DefId>,
    disabled_mods: Vec<DefId>,
    max_concurrent_io: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mods_dir: "".into(),
            enabled_mods: Vec::new(),
            disabled_mods: Vec::new(),
            max_concurrent_io: 10,
        }
    }
}
