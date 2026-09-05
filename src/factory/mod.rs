use std::{fs, path::PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    item::ItemDef,
    modding::{
        DefHandle, DefId, Definition, DefinitionLoadError, ModLoadState, Registry, ResolvedRegistry,
    },
};

pub struct FactoryPlugin;

impl Plugin for FactoryPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Debug)]
pub struct MachineDef {
    recipe_kinds: Vec<DefId>,
}

impl Definition for MachineDef {
    const DIR: &'static str = "machines";

    async fn load(mod_id: DefId, path: PathBuf) -> Result<(DefId, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawMachineDef {
            id: DefId,
            recipe_kinds: Vec<DefId>,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawMachineDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let id = mod_id.join(raw.id);

        Ok((
            id,
            MachineDef {
                recipe_kinds: raw.recipe_kinds,
            },
        ))
    }
}

#[derive(Debug)]
pub struct MachineDefResolved {
    recipe_kinds: Vec<DefHandle<RecipeKindDef>>,
}

#[derive(Debug, Component)]
pub struct Machine {
    handle: DefHandle<MachineDef>,
    recipe: DefHandle<RecipeDef>,
}

#[derive(Debug)]
pub struct RecipeDef {
    kind: DefId,
    inputs: Vec<(DefId, usize)>,
    outputs: Vec<(DefId, usize)>,
    time: f32,
}

impl RecipeDef {
    fn resolve(
        mut commands: Commands,
        registry: Res<Registry<RecipeDef>>,
        recipe_kinds: Res<Registry<RecipeKindDef>>,
        items: Res<Registry<ItemDef>>,
    ) {
        let resolved = ResolvedRegistry::new(registry.iter().map(|def| {
            let kind = recipe_kinds.get_handle(&def.kind)?;
            let inputs = def
                .inputs
                .iter()
                .map(|(id, num)| Some((items.get_handle(id)?, *num)))
                .collect::<Option<Vec<_>>>()?;
            let outputs = def
                .outputs
                .iter()
                .map(|(id, num)| Some((items.get_handle(id)?, *num)))
                .collect::<Option<Vec<_>>>()?;
            Some(ResolvedRecipe {
                kind,
                inputs,
                outputs,
                time: def.time,
            })
        }));

        commands.insert_resource(resolved);
    }
}

impl Definition for RecipeDef {
    const DIR: &'static str = "recipes";

    async fn load(mod_id: DefId, path: PathBuf) -> Result<(DefId, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawRecipeDef {
            id: DefId,
            kind: DefId,
            inputs: Vec<(DefId, usize)>,
            outputs: Vec<(DefId, usize)>,
            time: f32,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawRecipeDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let id = mod_id.join(raw.id);

        Ok((
            id,
            RecipeDef {
                kind: raw.kind,
                inputs: raw.inputs,
                outputs: raw.outputs,
                time: raw.time,
            },
        ))
    }

    fn build(app: &mut App) {
        app.add_systems(OnEnter(ModLoadState::Resolve), Self::resolve);
    }
}

#[derive(Debug)]
pub struct ResolvedRecipe {
    kind: DefHandle<RecipeKindDef>,
    inputs: Vec<(DefHandle<ItemDef>, usize)>,
    outputs: Vec<(DefHandle<ItemDef>, usize)>,
    time: f32,
}

#[derive(Debug)]
pub struct RecipeKindDef {}

impl Definition for RecipeKindDef {
    const DIR: &'static str = "recipe_kinds";

    async fn load(mod_id: DefId, path: PathBuf) -> Result<(DefId, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawRecipeKindDef {
            id: DefId,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawRecipeKindDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let id = mod_id.join(raw.id);

        Ok((id, RecipeKindDef {}))
    }
}
