use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::modding::{DefHandle, DefId, Definition, DefinitionLoadError};

#[derive(Debug)]
pub struct TileDef {
    pub sprite_path: String,
    pub friction: f32,
    pub restitution: f32,
}

impl Definition for TileDef {
    const DIR: &'static str = "tiles";

    async fn load(mod_id: DefId, path: PathBuf) -> Result<(DefId, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawTileDef {
            id: DefId,
            sprite_path: String,
            friction: f32,
            restitution: f32,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawTileDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let id = mod_id.join(raw.id);

        Ok((
            id,
            TileDef {
                sprite_path: raw.sprite_path,
                friction: raw.friction,
                restitution: raw.restitution,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub handle: DefHandle<TileDef>,
}

impl Tile {
    pub fn new(handle: DefHandle<TileDef>) -> Self {
        Self { handle }
    }
}
