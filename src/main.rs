#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    camera::ScalingMode,
    prelude::*,
    window::{PresentMode, WindowMode, WindowResolution},
};

use necrofactory::{
    GameState,
    debug::{
        DebugPlugin, coord::CoordinateDebugPlugin, jetpack::JetPackDebugPlugin,
        physics::PhysicsDebugPlugin,
    },
    graphics::GraphicsPlugin,
    input::{InputAction, InputPlugin, InputState, WorldCursor},
    modding::{Id, ModAssetSourcePlugin, ModPlugin, Registry},
    physics::{Collider, Drag, Mass, PhysicsPlugin, Restitution, Rigidbody},
    player::{FuelTank, JetPackPlugin, Jetpack, JetpackControl, Player},
    rand::RandPlugin,
    world::{BaseChunk, RebaseSet, World, WorldPlugin, WorldTransform},
    world_gen::WorldGenPlugin,
};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            ModAssetSourcePlugin,
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Necrofactory".to_owned(),
                        #[cfg(debug_assertions)]
                        mode: WindowMode::Windowed,
                        #[cfg(debug_assertions)]
                        resolution: WindowResolution::new(1920, 1080),
                        #[cfg(not(debug_assertions))]
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                        present_mode: PresentMode::AutoVsync,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            ModPlugin,
            RandPlugin { seed: 1 },
            WorldPlugin,
            WorldGenPlugin,
            GraphicsPlugin,
            InputPlugin,
            DebugPlugin,
            CoordinateDebugPlugin,
            PhysicsDebugPlugin,
            JetPackDebugPlugin,
            PhysicsPlugin,
            JetPackPlugin,
        ))
        .insert_state(GameState::Boot)
        .add_systems(OnEnter(GameState::Boot), boot)
        .add_systems(OnEnter(GameState::InGame), setup)
        .add_systems(OnExit(GameState::InGame), cleanup)
        .add_systems(Update, (esc_exit, mod_reload))
        .add_systems(
            Update,
            (zoom, toggle_tile).run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            Update,
            (player_move, player_follow).run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            PostUpdate,
            camera_follow
                .after(RebaseSet)
                .run_if(in_state(GameState::InGame)),
        )
        .run()
}

fn boot(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::ModLoading);
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 256.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    commands.spawn((
        Player,
        WorldTransform::default(),
        Sprite::from_color(Color::hsv(0.0, 1.0, 0.4), Vec2 { x: 16.0, y: 16.0 }),
        Rigidbody,
        Mass(1.0),
        Collider(Vec2::ONE),
        Drag(0.002),
        Restitution(0.1),
        Jetpack {
            fuel_use_rate: 20.0,
            force: 20.0,
        },
        FuelTank {
            fuel: 100.0,
            max_fuel: 100.0,
            fuel_regen_rate: 100.0,
        },
    ));
}

fn cleanup(
    mut commands: Commands,
    player: Query<Entity, With<Player>>,
    camera: Query<Entity, With<Camera2d>>,
) {
    commands.entity(player.single().unwrap()).despawn();
    commands.entity(camera.single().unwrap()).despawn();
}

fn camera_follow(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    camera.single_mut().unwrap().translation = player.single().unwrap().translation;
}

fn player_follow(player: Query<&WorldTransform, With<Player>>, mut base: ResMut<BaseChunk>) {
    let world_transform = player.single().unwrap();
    let chunk = world_transform.translation.chunk();
    if chunk != base.0 {
        base.0 = chunk;
    }
}

fn player_move(
    mut acceleration: Query<&mut JetpackControl, With<Player>>,
    input: Res<InputState>,
    registry: Res<Registry<InputAction>>,
) {
    let mut jetpack_control = acceleration.single_mut().unwrap();

    let left = registry.lookup("base::left").unwrap();
    let right = registry.lookup("base::right").unwrap();
    let fast = registry.lookup("base::fast").unwrap();
    let up = registry.lookup("base::up").unwrap();
    let down = registry.lookup("base::down").unwrap();
    let hover = registry.lookup("base::hover").unwrap();

    let fast = input.pressed(fast) as i32 as f32 + 1.0;

    jetpack_control.hover ^= input.just_pressed(hover);

    jetpack_control.throttle = input.vec2(right, left, up, down) * fast * 0.25;
    if !jetpack_control.hover {
        jetpack_control.throttle.y += input.pressed(up) as i32 as f32 * 0.75;
    }
}

fn zoom(
    mut camera: Query<&mut Projection, With<Camera>>,
    input: Res<InputState>,
    registry: Res<Registry<InputAction>>,
) {
    let projection = camera.single_mut().unwrap().into_inner();
    if let Projection::Orthographic(projection) = projection {
        let zoom;
        if input.just_pressed(registry.lookup("base::zoom_in").unwrap()) {
            zoom = 1.0;
        } else if input.just_pressed(registry.lookup("base::zoom_out").unwrap()) {
            zoom = -1.0;
        } else {
            zoom = 0.0;
        }
        projection.scale *= 1.0 - zoom * 0.25;
    }
}

fn toggle_tile(
    mut world: ResMut<World>,
    input: Res<InputState>,
    cursor: Res<WorldCursor>,
    registry: Res<Registry<InputAction>>,
) {
    let Some(player_pos) = cursor.0 else {
        return;
    };
    if input.just_pressed(registry.lookup("base::toggle").unwrap()) {
        let tile = world.get_tile_mut(IVec2::from(player_pos).into()).unwrap();
        if tile.id == Id::ZERO {
            tile.id = Id::ONE;
        } else {
            tile.id = Id::ZERO;
        }
    }
}

fn mod_reload(input: Res<ButtonInput<KeyCode>>, mut state: ResMut<NextState<GameState>>) {
    if input.just_pressed(KeyCode::KeyR) {
        state.set(GameState::ModLoading);
    }
}

fn esc_exit(input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
