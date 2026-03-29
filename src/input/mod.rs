use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
};

use bevy::{
    input::{InputSystems, mouse::MouseWheel},
    prelude::*,
};

use serde::Deserialize;

use crate::{
    GameState,
    math::HybridVec2,
    modding::{DefPath, Definition, DefinitionLoadError, Id, ModInfo, ModLoadState, Registry},
    world::{BaseChunk, TILE_SIZE},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<InputBindings>()
            .init_resource::<Registry<InputAction>>()
            .init_resource::<WorldCursor>()
            .add_systems(OnEnter(ModLoadState::Finalize), setup_input_map)
            .add_systems(
                PreUpdate,
                (
                    button_input_system,
                    scroll_input_system,
                    cursor_input_system,
                    world_cursor_input_system.run_if(in_state(GameState::InGame)),
                )
                    .after(InputSystems),
            );
    }
}

fn setup_input_map(mut map: ResMut<InputBindings>, registry: Res<Registry<InputAction>>) {
    for (id, _path, input) in registry.iter_with_id() {
        map.insert(id, input.default.clone());
    }
}

fn button_input_system(
    mut state: ResMut<InputState>,
    map: Res<InputBindings>,
    key_buttons: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    state.clear();

    for (&id, input) in map.bindings.iter() {
        // Check if all modifiers are pressed
        // Sorry for the monstrous if statement
        // TODO: Fix (maybe???)
        let mods_pressed = (!input.shift
            || key_buttons.pressed(KeyCode::ShiftLeft)
            || key_buttons.pressed(KeyCode::ShiftRight))
            && (!input.control
                || key_buttons.pressed(KeyCode::ControlLeft)
                || key_buttons.pressed(KeyCode::ControlRight))
            && (!input.alt
                || key_buttons.pressed(KeyCode::AltLeft)
                || key_buttons.pressed(KeyCode::AltRight));

        match input.kind {
            InputBindingKind::None => {}
            InputBindingKind::KeyButton(key_code) => {
                if key_buttons.just_pressed(key_code) && mods_pressed {
                    state.press(id);
                } else if key_buttons.just_released(key_code) {
                    state.release(id);
                }
            }
            InputBindingKind::MouseButton(mouse_button) => {
                if mouse_buttons.just_pressed(mouse_button) && mods_pressed {
                    state.press(id);
                } else if mouse_buttons.just_released(mouse_button) {
                    state.release(id);
                }
            }
        }
    }
}

fn scroll_input_system(mut state: ResMut<InputState>, mut scroll: MessageReader<MouseWheel>) {
    state.scroll = scroll.read().fold(0.0, |sum, event| sum + event.y);
}

fn cursor_input_system(mut state: ResMut<InputState>, windows: Query<&Window>) {
    state.cursor = windows.single().ok().and_then(|w| w.cursor_position());
}

fn world_cursor_input_system(
    mut world_cursor: ResMut<WorldCursor>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    base: Res<BaseChunk>,
) {
    fn f(
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        base: Res<BaseChunk>,
    ) -> Option<HybridVec2> {
        let window = windows.single().ok()?;
        let (camera, camera_transform) = camera.single().ok()?;

        let cursor_pos = window.cursor_position()?;
        let world_pos = camera
            .viewport_to_world_2d(camera_transform, cursor_pos)
            .ok()?;
        let tile_pos = world_pos / TILE_SIZE as f32;
        let chunk_pos = base.0;
        Some(HybridVec2::from_chunk_tile(chunk_pos, tile_pos).round())
    }

    world_cursor.0 = f(windows, camera, base);
}

#[derive(Debug, Default, Resource)]
pub struct WorldCursor(pub Option<HybridVec2>);

#[derive(Debug, Default, Resource)]
pub struct InputState {
    pressed: HashSet<Id<InputAction>>,
    just_pressed: HashSet<Id<InputAction>>,
    just_released: HashSet<Id<InputAction>>,
    cursor: Option<Vec2>,
    scroll: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            cursor: None,
            scroll: 0.0,
        }
    }

    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn press(&mut self, id: Id<InputAction>) {
        if self.pressed.insert(id) {
            self.just_pressed.insert(id);
        }
    }

    pub fn release(&mut self, id: Id<InputAction>) {
        if self.pressed.remove(&id) {
            self.just_released.insert(id);
        }
    }

    pub fn pressed(&self, id: Id<InputAction>) -> bool {
        self.pressed.contains(&id)
    }

    pub fn just_pressed(&self, id: Id<InputAction>) -> bool {
        self.just_pressed.contains(&id)
    }

    pub fn just_released(&self, id: Id<InputAction>) -> bool {
        self.just_released.contains(&id)
    }

    pub fn axis(&self, positive: Id<InputAction>, negative: Id<InputAction>) -> f32 {
        let positive = self.pressed.contains(&positive) as i8;
        let negative = self.pressed.contains(&negative) as i8;
        (positive - negative) as f32
    }

    pub fn vec2(
        &self,
        positive_x: Id<InputAction>,
        negative_x: Id<InputAction>,
        positive_y: Id<InputAction>,
        negative_y: Id<InputAction>,
    ) -> Vec2 {
        let x = self.axis(positive_x, negative_x);
        let y = self.axis(positive_y, negative_y);
        Vec2::new(x, y)
    }

    pub fn cursor(&self) -> Option<Vec2> {
        self.cursor
    }

    pub fn scroll(&self) -> f32 {
        self.scroll
    }
}

#[derive(Debug, Default, Resource)]
pub struct InputBindings {
    bindings: HashMap<Id<InputAction>, InputBinding>,
}

impl InputBindings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn get(&self, id: Id<InputAction>) -> Option<&InputBinding> {
        self.bindings.get(&id)
    }

    pub fn insert(&mut self, id: Id<InputAction>, input: InputBinding) {
        self.bindings.insert(id, input);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct InputBinding {
    pub kind: InputBindingKind,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

impl InputBinding {
    pub const SHIFT: u8 = 0b001;
    pub const CONTROL: u8 = 0b010;
    pub const ALT: u8 = 0b100;

    pub fn from_kind(input_type: InputBindingKind) -> Self {
        Self {
            kind: input_type,
            shift: false,
            control: false,
            alt: false,
        }
    }

    pub fn none() -> Self {
        Self::from_kind(InputBindingKind::None)
    }

    pub fn key(key_code: KeyCode) -> Self {
        Self::from_kind(InputBindingKind::KeyButton(key_code))
    }

    pub fn mouse(mouse_button: MouseButton) -> Self {
        Self::from_kind(InputBindingKind::MouseButton(mouse_button))
    }

    pub fn with_modifiers(mut self, shift: bool, ctrl: bool, alt: bool) -> Self {
        self.shift = shift;
        self.control = ctrl;
        self.alt = alt;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_ctrl(mut self) -> Self {
        self.control = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }
}

impl From<InputBindingKind> for InputBinding {
    fn from(value: InputBindingKind) -> Self {
        Self::from_kind(value)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum InputBindingKind {
    #[default]
    None,
    KeyButton(KeyCode),
    MouseButton(MouseButton),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct InputAction {
    pub name: String,
    pub default: InputBinding,
}

impl InputAction {
    pub fn new(name: &str, default: InputBinding) -> Self {
        Self {
            name: name.to_string(),
            default,
        }
    }

    pub fn key(name: &str, default: KeyCode) -> Self {
        Self::new(name, InputBinding::key(default))
    }

    pub fn mouse(name: &str, default: MouseButton) -> Self {
        Self::new(name, InputBinding::mouse(default))
    }

    pub(crate) fn nameless(default: InputBinding) -> Self {
        Self::new("", default)
    }
}

impl Definition for InputAction {
    const DIR: &'static str = "inputs";

    async fn load(
        mod_info: ModInfo,
        path: PathBuf,
    ) -> Result<(DefPath, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawInputAction {
            path: DefPath,
            name: String,
            default: InputBinding,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawInputAction = ron::from_str(&string)?;

        let def_path = mod_info.id().join(raw.path);

        Ok((
            def_path,
            InputAction {
                name: raw.name,
                default: raw.default,
            },
        ))
    }
}

impl From<InputBinding> for InputAction {
    fn from(value: InputBinding) -> Self {
        Self::nameless(value)
    }
}
