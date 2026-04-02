use std::{fs, path::PathBuf};

use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    item::ItemDef,
    modding::{
        DefPath, Definition, DefinitionLoadError, Id, ModInfo, Registry, ResolutionError, Resolve,
        resolve,
    },
};

pub struct FactoryPlugin;

impl Plugin for FactoryPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Debug)]
pub struct MachineDef {
    recipe_kinds: Vec<DefPath>,
}

impl Definition for MachineDef {
    const DIR: &'static str = "machines";

    async fn load(
        mod_info: ModInfo,
        path: PathBuf,
    ) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawMachineDef {
            path: DefPath,
            recipe_kinds: Vec<DefPath>,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawMachineDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_info.id().join(raw.path);

        Ok((
            def_path,
            MachineDef {
                recipe_kinds: raw.recipe_kinds,
            },
        ))
    }
}

impl Resolve for MachineDef {
    type Output = MachineDefResolved;

    fn resolve(&self, world: &World) -> Result<Self::Output, ResolutionError> {
        let registry = world.get_resource::<Registry<RecipeKindDef>>().unwrap();

        let mut recipe_kinds = Vec::with_capacity(self.recipe_kinds.len());
        for kind in &self.recipe_kinds {
            let id = resolve(registry, kind)?;
            recipe_kinds.push(id);
        }

        Ok(MachineDefResolved { recipe_kinds })
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

impl Definition for RecipeDef {
    const DIR: &'static str = "recipes";

    async fn load(
        mod_info: ModInfo,
        path: PathBuf,
    ) -> Result<(DefPath, Self), DefinitionLoadError> {
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

        let def_path = mod_info.id().join(raw.path);

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
}

impl Resolve for RecipeDef {
    type Output = Recipe;

    fn resolve(&self, world: &World) -> Result<Self::Output, ResolutionError> {
        let recipe_kinds = world.get_resource::<Registry<RecipeKindDef>>().unwrap();
        let items = world.get_resource::<Registry<ItemDef>>().unwrap();

        let kind = resolve(recipe_kinds, &self.kind)?;

        let mut inputs = Vec::with_capacity(self.inputs.len());
        for (item, count) in &self.inputs {
            let id = resolve(items, item)?;
            inputs.push((id, *count));
        }

        let mut outputs = Vec::with_capacity(self.outputs.len());
        for (item, count) in &self.outputs {
            let id = resolve(items, item)?;
            outputs.push((id, *count));
        }

        Ok(Recipe {
            kind,
            inputs,
            outputs,
            time: self.time,
        })
    }
}

#[derive(Debug)]
pub struct Recipe {
    kind: Id<RecipeKindDef>,
    inputs: Vec<(Id<ItemDef>, usize)>,
    outputs: Vec<(Id<ItemDef>, usize)>,
    time: f32,
}

#[derive(Debug)]
pub struct RecipeKindDef {}

impl Definition for RecipeKindDef {
    const DIR: &'static str = "recipe_kinds";

    async fn load(
        mod_info: ModInfo,
        path: PathBuf,
    ) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawRecipeKindDef {
            path: DefPath,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawRecipeKindDef = ron::from_str(&string).map_err(|e| (e, path))?;

        let def_path = mod_info.id().join(raw.path);

        Ok((def_path, RecipeKindDef {}))
    }
}
