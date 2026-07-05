use std::{fs, path::PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::modding::{DefPath, Definition, DefinitionLoadError};

#[derive(Debug)]
pub struct ItemDef {
    pub name: String,
    pub sprite: PathBuf,
}

impl Definition for ItemDef {
    const DIR: &'static str = "items";

    async fn load(mod_id: DefPath, path: PathBuf) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawItemDef {
            path: DefPath,
            name: String,
            sprite: PathBuf,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawItemDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_id.join(raw.path);

        Ok((
            def_path,
            ItemDef {
                name: raw.name,
                sprite: raw.sprite,
            },
        ))
    }
}
