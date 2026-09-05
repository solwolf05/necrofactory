use bevy::prelude::*;

use crate::modding::{DefHandle, ModLoadState};

pub fn cleanup(mut next_state: ResMut<NextState<ModLoadState>>) {
    info!("Mod resolution complete");
    next_state.set(ModLoadState::LoadAssets);
}

#[derive(Debug, Default, Resource)]
pub struct ResolvedRegistry<R>
where
    R: Send + Sync,
{
    definitions: Vec<Option<R>>,
}

impl<R> ResolvedRegistry<R>
where
    R: Send + Sync,
{
    pub fn new(defs: impl IntoIterator<Item = Option<R>>) -> Self {
        Self {
            definitions: defs.into_iter().collect(),
        }
    }

    /// Retrieves the definition associated with the given ID.
    pub fn get(&self, handle: DefHandle<R>) -> Option<&R> {
        self.definitions.get(handle.to_index()).flatten_ref()
    }
}
