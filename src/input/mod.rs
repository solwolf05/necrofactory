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
    modding::{DefHandle, DefId, Definition, DefinitionLoadError, ModLoadState, Registry},
    world::{BaseChunk, TILE_SIZE},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<Scroll>()
            .init_resource::<Cursor>()
            .init_resource::<InputMap>()
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

fn setup_input_map(mut map: ResMut<InputMap>, registry: Res<Registry<InputAction>>) {
    for (handle, input) in registry.iter_with_handle() {
        map.insert(handle, input.default.clone());
    }
}

fn button_input_system(
    mut state: ResMut<InputState>,
    map: Res<InputMap>,
    key_buttons: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    state.clear();

    for (&handle, input) in map.map.iter() {
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
            InputKind::None => {}
            InputKind::KeyButton(key_code) => {
                if key_buttons.just_pressed(key_code) && mods_pressed {
                    state.press(handle);
                } else if key_buttons.just_released(key_code) {
                    state.release(handle);
                }
            }
            InputKind::MouseButton(mouse_button) => {
                if mouse_buttons.just_pressed(mouse_button) && mods_pressed {
                    state.press(handle);
                } else if mouse_buttons.just_released(mouse_button) {
                    state.release(handle);
                }
            }
        }
    }
}

fn scroll_input_system(mut scroll: ResMut<Scroll>, mut events: MessageReader<MouseWheel>) {
    scroll.0 = events.read().fold(0.0, |sum, event| sum + event.y);
}

fn cursor_input_system(mut cursor: ResMut<Cursor>, windows: Query<&Window>) {
    cursor.0 = windows.single().ok().and_then(|w| w.cursor_position());
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
pub struct InputState {
    pressed: HashSet<DefHandle<InputAction>>,
    just_pressed: HashSet<DefHandle<InputAction>>,
    just_released: HashSet<DefHandle<InputAction>>,
}

#[derive(Debug, Default, Resource, Deref)]
pub struct WorldCursor(pub Option<HybridVec2>);

#[derive(Debug, Default, Resource, Deref)]
pub struct Cursor(pub Option<Vec2>);

#[derive(Debug, Default, Resource, Deref)]
pub struct Scroll(pub f32);

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    pub fn clear(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn press(&mut self, handle: DefHandle<InputAction>) {
        if self.pressed.insert(handle) {
            self.just_pressed.insert(handle);
        }
    }

    pub fn release(&mut self, handle: DefHandle<InputAction>) {
        if self.pressed.remove(&handle) {
            self.just_released.insert(handle);
        }
    }

    pub fn pressed(&self, handle: DefHandle<InputAction>) -> bool {
        self.pressed.contains(&handle)
    }

    pub fn just_pressed(&self, handle: DefHandle<InputAction>) -> bool {
        self.just_pressed.contains(&handle)
    }

    pub fn just_released(&self, handle: DefHandle<InputAction>) -> bool {
        self.just_released.contains(&handle)
    }

    pub fn axis(&self, positive: DefHandle<InputAction>, negative: DefHandle<InputAction>) -> f32 {
        let positive = self.pressed.contains(&positive) as i8;
        let negative = self.pressed.contains(&negative) as i8;
        (positive - negative) as f32
    }

    pub fn vec2(
        &self,
        positive_x: DefHandle<InputAction>,
        negative_x: DefHandle<InputAction>,
        positive_y: DefHandle<InputAction>,
        negative_y: DefHandle<InputAction>,
    ) -> Vec2 {
        let x = self.axis(positive_x, negative_x);
        let y = self.axis(positive_y, negative_y);
        Vec2::new(x, y)
    }
}

#[derive(Debug, Default, Resource)]
pub struct InputMap {
    map: HashMap<DefHandle<InputAction>, PhysicalInput>,
}

impl InputMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, handle: DefHandle<InputAction>) -> Option<&PhysicalInput> {
        self.map.get(&handle)
    }

    pub fn insert(&mut self, handle: DefHandle<InputAction>, input: PhysicalInput) {
        self.map.insert(handle, input);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct PhysicalInput {
    pub kind: InputKind,
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

impl PhysicalInput {
    pub fn from_kind(input_type: InputKind) -> Self {
        Self {
            kind: input_type,
            shift: false,
            control: false,
            alt: false,
        }
    }

    pub fn none() -> Self {
        Self::from_kind(InputKind::None)
    }

    pub fn key(key_code: KeyCode) -> Self {
        Self::from_kind(InputKind::KeyButton(key_code))
    }

    pub fn mouse(mouse_button: MouseButton) -> Self {
        Self::from_kind(InputKind::MouseButton(mouse_button))
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

impl From<InputKind> for PhysicalInput {
    fn from(value: InputKind) -> Self {
        Self::from_kind(value)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum InputKind {
    #[default]
    None,
    KeyButton(KeyCode),
    MouseButton(MouseButton),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
pub struct InputAction {
    pub name: String,
    pub default: PhysicalInput,
}

impl InputAction {
    pub fn new(name: String, default: PhysicalInput) -> Self {
        Self { name, default }
    }

    pub fn key(name: String, default: KeyCode) -> Self {
        Self::new(name, PhysicalInput::key(default))
    }

    pub fn mouse(name: String, default: MouseButton) -> Self {
        Self::new(name, PhysicalInput::mouse(default))
    }
}

impl Definition for InputAction {
    const DIR: &'static str = "inputs";

    async fn load(mod_id: DefId, path: PathBuf) -> Result<(DefId, Self), DefinitionLoadError> {
        #[derive(Deserialize)]
        struct RawInputAction {
            id: DefId,
            name: String,
            default: PhysicalInput,
        }

        let string = fs::read_to_string(&path)?;
        let raw: RawInputAction = ron::from_str(&string).map_err(|e| (e, path))?;

        let id = mod_id.join(raw.id);

        Ok((
            id,
            InputAction {
                name: raw.name,
                default: raw.default,
            },
        ))
    }
}
