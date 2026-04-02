use std::{
    error::Error,
    fmt::{Debug, Display},
};

use bevy::prelude::*;

use crate::modding::{DefPath, Definition, Id, ModLoadState, Registry};

pub fn resolve_registry<D: Resolve>(world: &mut World) {
    let registry = world.resource::<Registry<D>>();
    let definitions = registry
        .iter()
        .map(|(path, definition)| {
            let id = definition.resolve(world);
            id.map_err(|err| {
                error!("{} for \"{}\"", err, path);
                err
            })
            .ok()
        })
        .collect::<Vec<_>>();

    let mut resolved = world.resource_mut::<ResolvedRegistry<D>>();
    resolved.definitions = definitions;
}

pub fn cleanup(mut next_state: ResMut<NextState<ModLoadState>>) {
    info!("Mod resolution complete");
    next_state.set(ModLoadState::LoadAssets);
}

pub trait Resolve: Definition {
    type Output: Send + Sync + Debug;

    fn resolve(&self, world: &World) -> Result<Self::Output, ResolutionError>;
}

pub fn resolve<D>(registry: &Registry<D>, path: &DefPath) -> Result<Id<D>, ResolutionError> {
    registry.lookup(path).ok_or(ResolutionError(path.clone()))
}

#[derive(Debug, Resource)]
pub struct ResolvedRegistry<D: Resolve> {
    definitions: Vec<Option<D::Output>>,
}

impl<D: Resolve> Default for ResolvedRegistry<D> {
    fn default() -> Self {
        Self {
            definitions: Default::default(),
        }
    }
}

impl<D: Resolve> ResolvedRegistry<D> {
    pub fn get(&self, id: Id<D>) -> Option<&D::Output> {
        self.definitions.get(id.index()).flatten_ref()
    }
}

#[derive(Debug)]
pub struct ResolutionError(pub DefPath);

impl Error for ResolutionError {}

impl Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to resolve \"{}\"", self.0)
    }
}
