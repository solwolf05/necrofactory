use std::{
    collections::VecDeque,
    error::Error,
    fmt::Display,
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
    time::Instant,
};

use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};

use crate::modding::{
    Definition, Id, ModInfo, ModLoadState, ModRegistry,
    types::{DefPath, Registry},
};

const MAX_CONCURRENT_IO: usize = 10;

#[derive(Debug, Default, Resource)]
pub struct TotalPending(pub usize);

#[derive(Debug, Resource)]
pub struct Pending<D: Definition>(pub VecDeque<(Id<ModInfo>, PathBuf)>, PhantomData<D>);

impl<D: Definition> Default for Pending<D> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

#[derive(Debug, Default, Resource)]
pub struct TotalActive(pub usize);

#[derive(Debug, Resource)]
pub struct Active<D: Definition>(pub Vec<Task<Result<(DefPath, D), DefinitionLoadError>>>);

impl<D: Definition> Default for Active<D> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Debug, Default, Resource)]
pub struct TotalComplete(pub usize);

#[derive(Debug, Resource)]
pub struct Complete<D: Definition>(pub usize, PhantomData<D>);

impl<D: Definition> Default for Complete<D> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

#[derive(Debug, Resource)]
pub struct ModRegistrationTime(Instant);

pub fn setup(mut commands: Commands) {
    commands.insert_resource(ModRegistrationTime(Instant::now()));
    commands.init_resource::<TotalPending>();
    commands.init_resource::<TotalActive>();
    commands.init_resource::<TotalComplete>();
}

pub fn cleanup(mut commands: Commands, time: Res<ModRegistrationTime>) {
    info!(
        "Mod registration complete ({}ms)",
        time.0.elapsed().as_millis_f32()
    );
    commands.remove_resource::<ModRegistrationTime>();
    commands.remove_resource::<TotalPending>();
    commands.remove_resource::<TotalActive>();
    commands.remove_resource::<TotalComplete>();
}

pub fn log(pending: Res<TotalPending>, active: Res<TotalActive>, complete: Res<TotalComplete>) {
    let total = pending.0 + active.0 + complete.0;

    info!(
        "{} / {} ({}%)",
        complete.0,
        total,
        complete.0 * 100 / (total)
    )
}

pub fn discover<D: Definition>(
    mut commands: Commands,
    mods: Res<ModRegistry>,
    mut total_pending: ResMut<TotalPending>,
) {
    let definitions: VecDeque<(Id<ModInfo>, PathBuf)> = mods
        .iter_enabled_with_id()
        .flat_map(|(id, _, mod_info)| read_mod_dir(id, mod_info, D::DIR))
        .collect();
    total_pending.0 += definitions.len();
    commands.insert_resource(Pending::<D>(definitions, PhantomData));
    commands.insert_resource(Active::<D>::default());
    commands.insert_resource(Complete::<D>::default());
}

fn read_mod_dir(id: Id<ModInfo>, mod_info: &ModInfo, path: &str) -> Vec<(Id<ModInfo>, PathBuf)> {
    let path: &Path = &mod_info.path.join(path);
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| (id, entry.path()))
        .collect()
}

pub fn clear<D: Definition>(mut registry: ResMut<Registry<D>>) {
    registry.clear();
}

pub fn spawn<D: Definition>(
    mods: Res<ModRegistry>,
    mut pending: ResMut<Pending<D>>,
    mut active: ResMut<Active<D>>,
    mut total_pending: ResMut<TotalPending>,
    mut total_active: ResMut<TotalActive>,
) {
    let pool = IoTaskPool::get();

    while active.0.len() < MAX_CONCURRENT_IO
        && let Some((mod_id, path)) = pending.0.pop_front()
    {
        let mod_info = mods.get(mod_id).unwrap().clone();
        active.0.push(pool.spawn(D::load(mod_info, path)));

        total_pending.0 -= 1;
        total_active.0 += 1;
    }
}

pub fn poll<D: Definition>(
    mut active: ResMut<Active<D>>,
    mut complete: ResMut<Complete<D>>,
    mut registry: ResMut<Registry<D>>,
    mut total_active: ResMut<TotalActive>,
    mut total_complete: ResMut<TotalComplete>,
) {
    active.0.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            match result {
                Ok((path, def)) => {
                    complete.0 += 1;
                    registry.register(path, def);
                }
                Err(err) => error!("Failed to load definition: {}", err),
            }
            total_active.0 -= 1;
            total_complete.0 += 1;
            false
        } else {
            true
        }
    });
}

pub fn check_loaded(
    mut next_state: ResMut<NextState<ModLoadState>>,
    pending: Res<TotalPending>,
    active: Res<TotalActive>,
) {
    if pending.0 == 0 && active.0 == 0 {
        next_state.set(ModLoadState::LoadAssets);
    }
}

#[derive(Debug)]
pub enum DefinitionLoadError {
    Io(std::io::Error),
    Parse(ron::error::SpannedError),
}

impl Display for DefinitionLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionLoadError::Io(error) => error.fmt(f),
            DefinitionLoadError::Parse(error) => write!(f, "{}", error),
        }
    }
}

impl Error for DefinitionLoadError {}

impl From<std::io::Error> for DefinitionLoadError {
    fn from(err: std::io::Error) -> Self {
        DefinitionLoadError::Io(err)
    }
}

impl From<ron::error::SpannedError> for DefinitionLoadError {
    fn from(err: ron::error::SpannedError) -> Self {
        DefinitionLoadError::Parse(err)
    }
}
