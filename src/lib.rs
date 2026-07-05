#![feature(duration_millis_float)]
#![feature(option_reference_flattening)]
#![feature(iter_map_windows)]

use bevy::{ecs::resource::Resource, state::state::States};
use serde::{Deserialize, Serialize};

use crate::modding::DefPath;

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

#[derive(Debug, Default, Clone, Resource)]
pub struct Config {
    installed_mods: Vec<DefPath>,
    enabled_mods: Vec<DefPath>,
}

impl Serialize for Config {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct RawConfig<'a> {
            mods: Mods<'a>,
        }

        #[derive(Serialize)]
        struct Mods<'a> {
            installed: &'a [DefPath],
            enabled: &'a [DefPath],
        }

        let raw = RawConfig {
            mods: Mods {
                installed: &self.installed_mods,
                enabled: &self.enabled_mods,
            },
        };

        raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawConfig {
            mods: Mods,
        }

        #[derive(Deserialize)]
        struct Mods {
            installed: Vec<DefPath>,
            enabled: Vec<DefPath>,
        }

        let raw = RawConfig::deserialize(deserializer)?;
        Ok(Config {
            installed_mods: raw.mods.installed,
            enabled_mods: raw.mods.enabled,
        })
    }
}
