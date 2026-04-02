use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::{collections::HashMap, fs};

use bevy::{
    asset::io::{AssetSourceBuilder, AssetSourceId, file::FileAssetReader},
    prelude::*,
};

use serde::Deserialize;

use crate::{
    GameState,
    modding::{
        asset_loading::begin_asset_loading, discovery::discover_mods, finalization::finalize,
        validation::validate_mods,
    },
};

pub use asset_loading::TileSprites;
pub use registration::DefinitionLoadError;
pub use resolution::{ResolutionError, Resolve, ResolvedRegistry, resolve};
pub use types::*;

mod asset_loading;
mod discovery;
mod finalization;
mod registration;
mod resolution;
mod types;
mod validation;

pub use types::{DefPathSegment, Id, Registry};

/// Loads mods at the start of the game and registers their types in the registry.
pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_sub_state::<ModLoadState>()
            .init_resource::<ModRegistry>()
            .init_resource::<TileSprites>();

        app.add_systems(OnEnter(ModLoadState::Discover), discover_mods);

        app.add_systems(OnEnter(ModLoadState::Validate), validate_mods)
            .add_systems(OnExit(ModLoadState::Validate), check_mods);

        app.add_systems(OnEnter(ModLoadState::Register), registration::setup)
            .add_systems(
                Update,
                (registration::log, registration::check_loaded)
                    .run_if(in_state(ModLoadState::Register)),
            )
            .add_systems(OnExit(ModLoadState::Register), registration::cleanup);

        app.add_systems(
            Update,
            resolution::cleanup.run_if(in_state(ModLoadState::Resolve)),
        );

        app.add_systems(OnEnter(ModLoadState::LoadAssets), begin_asset_loading)
            .add_systems(
                Update,
                asset_loading::check_loaded.run_if(in_state(ModLoadState::LoadAssets)),
            )
            .add_systems(OnExit(ModLoadState::LoadAssets), asset_loading::cleanup);

        app.add_systems(OnEnter(ModLoadState::Finalize), finalize);
    }
}

pub struct ModAssetSourcePlugin;

impl Plugin for ModAssetSourcePlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            AssetSourceId::Name("mods".into()),
            AssetSourceBuilder::new(|| Box::new(FileAssetReader::new(mods_path()))),
        );
    }
}

pub struct DefinitionPlugin<D: Definition + Debug>(PhantomData<D>);

impl<D: Definition + Debug> Plugin for DefinitionPlugin<D> {
    fn build(&self, app: &mut App) {
        app.init_resource::<Registry<D>>()
            .add_systems(
                OnEnter(ModLoadState::Register),
                (
                    registration::discover::<D>.after(registration::setup),
                    registration::clear::<D>,
                ),
            )
            .add_systems(
                Update,
                (registration::spawn::<D>, registration::poll::<D>)
                    .run_if(in_state(ModLoadState::Register)),
            );
        #[cfg(debug_assertions)]
        app.add_systems(OnExit(ModLoadState::Register), registration::check::<D>);
    }
}

impl<D: Definition + Debug> Default for DefinitionPlugin<D> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub struct ResolveDefinitionPlugin<D: Definition + Debug>(PhantomData<D>);

impl<D: Resolve + Debug> Plugin for ResolveDefinitionPlugin<D> {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefinitionPlugin::<D>::default());
        app.init_resource::<ResolvedRegistry<D>>().add_systems(
            OnEnter(ModLoadState::Resolve),
            resolution::resolve_registry::<D>,
        );
    }
}

impl<D: Definition + Debug> Default for ResolveDefinitionPlugin<D> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(SubStates, Debug, Default, Clone, Eq, PartialEq, Hash)]
#[source(GameState = GameState::ModLoading)]
pub enum ModLoadState {
    #[default]
    Discover,
    Validate,
    Register,
    Resolve,
    LoadAssets,
    Finalize,
}

fn mods_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mods");
        if candidate.exists() {
            return candidate;
        }
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("mods")
}

fn check_mods(mods: Res<ModRegistry>) {
    info!("Mods:\n{}", *mods);
    info!(
        "Mod load order: {}",
        mods.load_order
            .iter()
            .map(|&id| mods.resolve(id).unwrap().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[derive(Default, Resource, Clone)]
pub struct ModRegistry {
    mods: Vec<(DefPathSegment, ModInfo)>,
    lookup: HashMap<DefPathSegment, Id<ModInfo>>,
    pub load_order: Vec<Id<ModInfo>>,
}

impl ModRegistry {
    pub fn register(&mut self, segment: DefPathSegment, mod_info: ModInfo) -> Id<ModInfo> {
        if let Some(id) = self.lookup.get(&segment).copied() {
            self.mods[id.index()].1 = mod_info;
            return id;
        }

        let id = Id::from_index(self.mods.len());
        self.mods.push((segment.clone(), mod_info));
        self.lookup.insert(segment, id);

        id
    }

    pub fn enable(&mut self, id: Id<ModInfo>) {
        if let Some(mod_info) = self.get_mut(id) {
            mod_info.enable();
        }
    }

    pub fn enable_segment(&mut self, segment: &DefPathSegment) {
        if let Some(mod_info) = self.get_by_segment_mut(segment) {
            mod_info.enable();
        }
    }

    pub fn disable(&mut self, id: Id<ModInfo>) {
        if let Some(mod_info) = self.get_mut(id) {
            mod_info.disable();
        }
    }

    pub fn disable_segment(&mut self, segment: &DefPathSegment) {
        if let Some(mod_info) = self.get_by_segment_mut(segment) {
            mod_info.disable();
        }
    }

    pub fn clear(&mut self) {
        self.mods.clear();
        self.lookup.clear();
        self.load_order.clear();
    }

    pub fn len(&self) -> usize {
        self.mods.len()
    }

    pub fn len_enabled(&self) -> usize {
        self.mods.iter().filter(|(_, m)| m.enabled()).count()
    }

    pub fn len_disabled(&self) -> usize {
        self.mods.iter().filter(|(_, m)| !m.enabled()).count()
    }

    pub fn lookup(&self, segment: &DefPathSegment) -> Option<Id<ModInfo>> {
        self.lookup.get(&segment).copied()
    }

    pub fn resolve(&self, id: Id<ModInfo>) -> Option<&DefPathSegment> {
        self.mods.get(id.index()).map(|r| &r.0)
    }

    pub fn get(&self, id: Id<ModInfo>) -> Option<&ModInfo> {
        self.mods.get(id.index()).map(|r| &r.1)
    }

    fn get_mut(&mut self, id: Id<ModInfo>) -> Option<&mut ModInfo> {
        self.mods.get_mut(id.index()).map(|r| &mut r.1)
    }

    pub fn get_by_segment(&self, segment: &DefPathSegment) -> Option<&ModInfo> {
        self.lookup(segment).and_then(|id| self.get(id))
    }

    pub fn get_by_segment_mut(&mut self, segment: &DefPathSegment) -> Option<&mut ModInfo> {
        self.lookup(segment).and_then(|id| self.get_mut(id))
    }

    pub fn contains(&self, id: Id<ModInfo>) -> bool {
        self.mods.len() > id.index()
    }

    pub fn contains_path(&self, segment: &DefPathSegment) -> bool {
        self.lookup.contains_key(segment)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DefPathSegment, &ModInfo)> {
        self.mods.iter().map(|(s, t)| (s, t))
    }

    pub fn iter_enabled(&self) -> impl Iterator<Item = (&DefPathSegment, &ModInfo)> {
        self.mods
            .iter()
            .filter(|(_, t)| t.enabled())
            .map(|(s, t)| (s, t))
    }

    pub fn iter_disabled(&self) -> impl Iterator<Item = (&DefPathSegment, &ModInfo)> {
        self.mods
            .iter()
            .filter(|(_, t)| !t.enabled())
            .map(|(s, t)| (s, t))
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (Id<ModInfo>, &DefPathSegment, &ModInfo)> {
        self.mods
            .iter()
            .enumerate()
            .map(|(i, (s, t))| (Id::from_index(i), s, t))
    }

    pub fn iter_enabled_with_id(
        &self,
    ) -> impl Iterator<Item = (Id<ModInfo>, &DefPathSegment, &ModInfo)> {
        self.mods
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| t.enabled())
            .map(|(i, (s, t))| (Id::from_index(i), s, t))
    }

    pub fn iter_disabled_with_id(
        &self,
    ) -> impl Iterator<Item = (Id<ModInfo>, &DefPathSegment, &ModInfo)> {
        self.mods
            .iter()
            .enumerate()
            .filter(|(_, (_, t))| !t.enabled())
            .map(|(i, (s, t))| (Id::from_index(i), s, t))
    }
}

impl Debug for ModRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, segment, mod_info) in self.iter_with_id() {
            writeln!(f, "{} {}: {:?}", id.get(), segment, mod_info)?;
        }
        Ok(())
    }
}

impl Display for ModRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (id, segment, mod_info) in self.iter_with_id() {
            writeln!(f, "{} {}: {}", id.get(), segment, mod_info)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModInfo {
    path: PathBuf,
    metadata: ModMetadata,
    enabled: bool,
}

impl ModInfo {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn id(&self) -> &DefPathSegment {
        &self.metadata.id
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn version(&self) -> &str {
        &self.metadata.version
    }

    pub fn author(&self) -> &str {
        &self.metadata.author
    }

    pub fn dependencies(&self) -> &HashMap<DefPathSegment, String> {
        &self.metadata.dependencies
    }

    pub fn optional_dependencies(&self) -> &HashMap<DefPathSegment, String> {
        &self.metadata.optional_dependencies
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        let _ = fs::remove_file(self.disable_path());
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        let _ = fs::write(self.disable_path(), []);
        self.enabled = false;
    }

    pub fn disable_path(&self) -> PathBuf {
        self.path.join("disabled")
    }
}

impl Display for ModInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} by {}", self.name(), self.version(), self.author())?;
        if self.dependencies().len() != 0 {
            write!(
                f,
                "; dependencies: {}",
                self.dependencies()
                    .into_iter()
                    .map(|(i, v)| format!("{} {}", i, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        if self.optional_dependencies().len() != 0 {
            write!(
                f,
                "; optional dependencies: {}",
                self.optional_dependencies()
                    .into_iter()
                    .map(|(i, v)| format!("{} {}", i, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            )?;
        }
        write!(
            f,
            "; path: mods/{}; {}",
            self.path().strip_prefix(mods_path()).unwrap().display(),
            if self.enabled() {
                "enabled"
            } else {
                "disabled"
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct ModMetadata {
    pub id: DefPathSegment,
    pub name: String,
    pub version: String,
    pub author: String,
    pub dependencies: HashMap<DefPathSegment, String>,
    pub optional_dependencies: HashMap<DefPathSegment, String>,
}

impl<'de> Deserialize<'de> for ModMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMetadata {
            pub id: DefPathSegment,
            pub name: String,
            pub version: String,
            pub author: String,
            pub dependencies: Option<HashMap<DefPathSegment, String>>,
            pub optional_dependencies: Option<HashMap<DefPathSegment, String>>,
        }

        let raw = RawMetadata::deserialize(deserializer)?;
        Ok(ModMetadata {
            id: raw.id,
            name: raw.name,
            version: raw.version,
            author: raw.author,
            dependencies: raw.dependencies.unwrap_or_default(),
            optional_dependencies: raw.optional_dependencies.unwrap_or_default(),
        })
    }
}

pub trait Definition: Sized + Send + Sync + 'static {
    const DIR: &'static str;

    fn load(
        mod_info: ModInfo,
        path: PathBuf,
    ) -> impl Future<Output = Result<(DefPath, Self), DefinitionLoadError>> + Send;
}
