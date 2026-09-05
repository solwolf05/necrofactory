use std::{fs, path::PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::modding::{DefId, Definition, DefinitionLoadError};

#[derive(Debug)]
pub struct ItemDef {
    pub name: String,
    pub sprite: PathBuf,
}

impl Definition for ItemDef {
    const DIR: &'static str = "items";

    async fn load(mod_id: DefId, path: PathBuf) -> Result<(DefId, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawItemDef {
            id: DefId,
            name: String,
            sprite: PathBuf,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawItemDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let id = mod_id.join(raw.id);

        Ok((
            id,
            ItemDef {
                name: raw.name,
                sprite: raw.sprite,
            },
        ))
    }
}
