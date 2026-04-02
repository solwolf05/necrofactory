use std::{fs, path::PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::modding::{DefPath, Definition, DefinitionLoadError, ModInfo};

#[derive(Debug)]
pub struct ItemDef {
    pub name: String,
    pub sprite: PathBuf,
}

impl Definition for ItemDef {
    const DIR: &'static str = "items";

    async fn load(
        mod_info: ModInfo,
        path: PathBuf,
    ) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawItemDef {
            path: DefPath,
            name: String,
            sprite: PathBuf,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawItemDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_info.id().join(raw.path);

        Ok((
            def_path,
            ItemDef {
                name: raw.name,
                sprite: raw.sprite,
            },
        ))
    }
}
