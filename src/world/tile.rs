use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::modding::{DefPath, Definition, DefinitionLoadError, Id};

#[derive(Debug)]
pub struct TileDef {
    pub sprite_path: String,
    pub friction: f32,
    pub restitution: f32,
}

impl Definition for TileDef {
    const DIR: &'static str = "tiles";

    async fn load(mod_id: DefPath, path: PathBuf) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawTileDef {
            path: DefPath,
            sprite_path: String,
            friction: f32,
            restitution: f32,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawTileDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_id.join(raw.path);

        Ok((
            def_path,
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
    pub id: Id<TileDef>,
}

impl Tile {
    pub fn new(id: Id<TileDef>) -> Self {
        Self { id }
    }
}
