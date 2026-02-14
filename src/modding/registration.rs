use std::{
    collections::VecDeque,
    error::Error,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};
use serde::Deserialize;

use crate::{
    input::{InputAction, InputBinding},
    modding::{
        ModInfo, ModLoadState, ModRegistry,
        types::{Path as DefPath, PathSegment, Registry},
    },
    world::tile::TileDef,
};

const MAX_CONCURRENT_IO: usize = 64;

#[derive(Debug, Default, Resource)]
pub struct Pending {
    pub inputs: VecDeque<(PathSegment, PathBuf)>,
    pub tiles: VecDeque<(PathSegment, PathBuf)>,
}

impl Pending {
    pub fn len(&self) -> usize {
        self.inputs.len() + self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty() && self.tiles.is_empty()
    }
}

#[derive(Debug, Default, Resource)]
pub struct Active {
    pub inputs: Vec<Task<Result<(DefPath, InputAction), DefinitionLoadError>>>,
    pub tiles: Vec<Task<Result<(DefPath, TileDef), DefinitionLoadError>>>,
}

impl Active {
    pub fn len(&self) -> usize {
        self.inputs.len() + self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty() && self.tiles.is_empty()
    }
}

#[derive(Debug, Default, Resource)]
pub struct Complete {
    pub inputs: usize,
    pub tiles: usize,
}

impl Complete {
    pub fn len(&self) -> usize {
        self.inputs + self.tiles
    }

    pub fn is_empty(&self) -> bool {
        self.inputs == 0 && self.tiles == 0
    }
}

pub fn discover_definitions(mods: Res<ModRegistry>, mut queue: ResMut<Pending>) {
    for mod_info in mods.iter() {
        queue.inputs.extend(read_mod_dir(mod_info, "inputs"));
        queue.tiles.extend(read_mod_dir(mod_info, "tiles"));
    }
}

fn read_mod_dir(mod_info: &ModInfo, path: &str) -> Vec<(PathSegment, PathBuf)> {
    read_dir(&mod_info.path.join(path))
        .map(|path| (mod_info.metadata.id.clone(), path))
        .collect()
}

fn read_dir(path: &Path) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
}

pub fn spawn_registration(
    mods: Res<ModRegistry>,
    mut pending: ResMut<Pending>,
    mut active: ResMut<Active>,
) {
    let pool = IoTaskPool::get();

    while active.len() < MAX_CONCURRENT_IO {
        if let Some((mod_id, path)) = pending.inputs.pop_front() {
            let mod_info = mods.mods.get(&mod_id).unwrap().clone();
            active.inputs.push(pool.spawn(load_input(mod_info, path)));
            continue;
        }

        if let Some((mod_id, path)) = pending.tiles.pop_front() {
            let mod_info = mods.mods.get(&mod_id).unwrap().clone();
            active.tiles.push(pool.spawn(load_tile(mod_info, path)));
            continue;
        }

        break;
    }
}

pub fn poll_registration(
    mut active: ResMut<Active>,
    mut complete: ResMut<Complete>,
    inputs: ResMut<Registry<InputAction>>,
    tiles: ResMut<Registry<TileDef>>,
) {
    poll_registry(
        &mut active.inputs,
        &mut complete.inputs,
        inputs.into_inner(),
    );
    poll_registry(&mut active.tiles, &mut complete.tiles, tiles.into_inner());
}

fn poll_registry<T>(
    tasks: &mut Vec<Task<Result<(DefPath, T), DefinitionLoadError>>>,
    complete: &mut usize,
    registry: &mut Registry<T>,
) {
    tasks.retain_mut(|task| {
        if let Some(result) = future::block_on(future::poll_once(task)) {
            match result {
                Ok((path, def)) => {
                    *complete += 1;
                    registry.register(path, def);
                }
                Err(err) => error!("Failed to load definition: {}", err),
            }
            false
        } else {
            true
        }
    });
}

pub fn check_registries_loaded(
    mut next_state: ResMut<NextState<ModLoadState>>,
    pending: Res<Pending>,
    active: Res<Active>,
) {
    if pending.is_empty() && active.is_empty() {
        info!("Mod registration complete");
        next_state.set(ModLoadState::LoadAssets);
    }
}

pub fn log_registration(pending: Res<Pending>, active: Res<Active>, complete: Res<Complete>) {
    let total = pending.len() + active.len() + complete.len();

    info!(
        "{} / {} ({}%)",
        complete.len(),
        total,
        complete.len() * 100 / (total)
    )
}

async fn load_input(
    mod_info: ModInfo,
    path: PathBuf,
) -> Result<(DefPath, InputAction), DefinitionLoadError> {
    #[derive(Deserialize)]
    struct RawInputAction {
        path: DefPath,
        name: String,
        default: InputBinding,
    }

    let string = fs::read_to_string(&path)?;
    let raw: RawInputAction = ron::from_str(&string)?;

    let def_path = mod_info.metadata.id.join(raw.path);

    Ok((
        def_path,
        InputAction {
            name: raw.name,
            default: raw.default,
        },
    ))
}

async fn load_tile(
    mod_info: ModInfo,
    path: PathBuf,
) -> Result<(DefPath, TileDef), DefinitionLoadError> {
    #[derive(Deserialize)]
    struct RawTileDef {
        path: DefPath,
        sprite_path: String,
    }

    let string = fs::read_to_string(&path)?;
    let raw: RawTileDef = ron::from_str(&string)?;

    let def_path = mod_info.metadata.id.join(raw.path);

    Ok((
        def_path,
        TileDef {
            sprite_path: raw.sprite_path,
        },
    ))
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
