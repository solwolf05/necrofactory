use std::{fs, path::PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    item::ItemDef,
    modding::{
        DefPath, Definition, DefinitionLoadError, Id, ModLoadState, Registry, ResolvedRegistry,
    },
};

pub struct FactoryPlugin;

impl Plugin for FactoryPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Debug)]
pub struct MachineDef {
    recipe_kinds: Vec<DefPath>,
}

impl Definition for MachineDef {
    const DIR: &'static str = "machines";

    async fn load(mod_id: DefPath, path: PathBuf) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawMachineDef {
            path: DefPath,
            recipe_kinds: Vec<DefPath>,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawMachineDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_id.join(raw.path);

        Ok((
            def_path,
            MachineDef {
                recipe_kinds: raw.recipe_kinds,
            },
        ))
    }
}

#[derive(Debug)]
pub struct MachineDefResolved {
    recipe_kinds: Vec<Id<RecipeKindDef>>,
}

#[derive(Debug, Component)]
pub struct Machine {
    id: Id<MachineDef>,
    recipe: Id<RecipeDef>,
}

#[derive(Debug)]
pub struct RecipeDef {
    kind: DefPath,
    inputs: Vec<(DefPath, usize)>,
    outputs: Vec<(DefPath, usize)>,
    time: f32,
}

impl RecipeDef {
    fn resolve(
        mut commands: Commands,
        registry: Res<Registry<RecipeDef>>,
        recipe_kinds: Res<Registry<RecipeKindDef>>,
        items: Res<Registry<ItemDef>>,
    ) {
        let resolved = ResolvedRegistry::new(registry.iter().map(|(_, def)| {
            let kind = recipe_kinds.lookup(&def.kind)?;
            let inputs = def
                .inputs
                .iter()
                .map(|(path, num)| Some((items.lookup(path)?, *num)))
                .collect::<Option<Vec<_>>>()?;
            let outputs = def
                .outputs
                .iter()
                .map(|(path, num)| Some((items.lookup(path)?, *num)))
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

    async fn load(mod_id: DefPath, path: PathBuf) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawRecipeDef {
            path: DefPath,
            kind: DefPath,
            inputs: Vec<(DefPath, usize)>,
            outputs: Vec<(DefPath, usize)>,
            time: f32,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawRecipeDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_id.join(raw.path);

        Ok((
            def_path,
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
    kind: Id<RecipeKindDef>,
    inputs: Vec<(Id<ItemDef>, usize)>,
    outputs: Vec<(Id<ItemDef>, usize)>,
    time: f32,
}

#[derive(Debug)]
pub struct RecipeKindDef {}

impl Definition for RecipeKindDef {
    const DIR: &'static str = "recipe_kinds";

    async fn load(mod_id: DefPath, path: PathBuf) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawRecipeKindDef {
            path: DefPath,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawRecipeKindDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_id.join(raw.path);

        Ok((def_path, RecipeKindDef {}))
    }
}
