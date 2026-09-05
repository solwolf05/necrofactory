use std::{
    any::type_name,
    collections::VecDeque,
    error::Error,
    fmt::{Debug, Display},
    fs,
    marker::PhantomData,
    path::{Path, PathBuf},
    time::Instant,
};

use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};

use crate::{
    Config,
    modding::{
        DefHandle, Definition, ModInfo, ModLoadState, ModRegistry,
        types::{DefId, Registry},
    },
};

#[derive(Debug, Default, Resource)]
pub struct TotalPending(pub usize);

#[derive(Debug, Resource)]
pub struct Pending<D: Definition>(pub VecDeque<(DefHandle<ModInfo>, PathBuf)>, PhantomData<D>);

impl<D: Definition> Default for Pending<D> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

#[derive(Debug, Default, Resource)]
pub struct TotalActive(pub usize);

#[derive(Debug, Resource)]
pub struct Active<D: Definition>(pub Vec<Task<Result<(DefId, D), DefinitionLoadError>>>);

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
        (complete.0 * 100).checked_div(total).unwrap_or(100)
    )
}

pub fn discover<D: Definition>(
    mut commands: Commands,
    mods: Res<ModRegistry>,
    mut total_pending: ResMut<TotalPending>,
) {
    let definitions: VecDeque<(DefHandle<ModInfo>, PathBuf)> = mods
        .iter_enabled_with_handle()
        .flat_map(|(handle, mod_info)| read_mod_dir(handle, mod_info, D::DIR))
        .collect();
    total_pending.0 += definitions.len();
    commands.insert_resource(Pending::<D>(definitions, PhantomData));
    commands.insert_resource(Active::<D>::default());
    commands.insert_resource(Complete::<D>::default());
}

fn read_mod_dir(
    handle: DefHandle<ModInfo>,
    mod_info: &ModInfo,
    path: &str,
) -> Vec<(DefHandle<ModInfo>, PathBuf)> {
    let path: &Path = &mod_info.path.join(path);
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| (handle, entry.path()))
        .filter(|(_, path)| path.extension().is_some_and(|ex| ex == "ron"))
        .collect()
}

pub fn clear<D: Definition>(mut registry: ResMut<Registry<D>>) {
    registry.clear();
}

pub fn spawn<D: Definition>(
    mods: Res<ModRegistry>,
    config: Res<Config>,
    mut pending: ResMut<Pending<D>>,
    mut active: ResMut<Active<D>>,
    mut total_pending: ResMut<TotalPending>,
    mut total_active: ResMut<TotalActive>,
) {
    let pool = IoTaskPool::get();

    while total_active.0 < config.max_concurrent_io
        && let Some((mod_id, path)) = pending.0.pop_front()
    {
        let id = mods.get(mod_id).unwrap().id();
        active.0.push(pool.spawn(D::load(id.clone(), path)));

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
                Ok((id, def)) => {
                    registry.register(id, def);
                }
                Err(err) => error!("Failed to load definition: {}", err),
            }
            complete.0 += 1;
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
        next_state.set(ModLoadState::Resolve);
    }
}

pub fn check<D: Definition + Debug>(registry: Res<Registry<D>>) {
    debug!("{}\n{:?}", type_name::<D>(), *registry);
}

#[derive(Debug)]
pub enum DefinitionLoadError {
    Io(std::io::Error),
    Parse(ron::error::SpannedError, PathBuf),
}

impl Display for DefinitionLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionLoadError::Io(error) => Display::fmt(error, f),
            DefinitionLoadError::Parse(error, path) => {
                write!(f, "{}: {}", path.display(), error)
            }
        }
    }
}

impl Error for DefinitionLoadError {}

impl From<std::io::Error> for DefinitionLoadError {
    fn from(err: std::io::Error) -> Self {
        DefinitionLoadError::Io(err)
    }
}

impl From<(ron::error::SpannedError, PathBuf)> for DefinitionLoadError {
    fn from((err, path): (ron::error::SpannedError, PathBuf)) -> Self {
        DefinitionLoadError::Parse(err, path)
    }
}
