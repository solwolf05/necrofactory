#![feature(duration_millis_float)]

use bevy::{camera::ScalingMode, prelude::*, window::WindowResolution};

use crate::{
    debug::DebugPlugin,
    graphics::GraphicsPlugin,
    input::{InputAction, InputPlugin, InputState},
    modding::{Id, ModAssetSourcePlugin, ModLoadState, ModPlugin, Registry, TileSprites},
    world::{BaseChunk, RebaseSet, TILE_SIZE, World, WorldPlugin, WorldPosition, WorldTransform},
    world_gen::WorldGenPlugin,
};

mod graphics;
mod input;
mod modding;
// mod serialization;
mod debug;
mod player;
mod world;
mod world_gen;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            ModAssetSourcePlugin,
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Modulus".to_owned(),
                        resolution: WindowResolution::new(1920, 1080),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            ModPlugin,
            WorldPlugin,
            WorldGenPlugin,
            GraphicsPlugin,
            InputPlugin,
            DebugPlugin,
        ))
        .insert_state(AppState::Boot)
        .add_systems(OnEnter(AppState::Boot), boot)
        .add_systems(OnEnter(AppState::InGame), setup)
        .add_systems(Update, esc_exit)
        .add_systems(
            Update,
            (zoom, toggle_tile).run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            Update,
            (player_move, player_follow).run_if(in_state(AppState::InGame)),
        )
        .add_systems(
            PostUpdate,
            camera_follow
                .after(RebaseSet)
                .run_if(in_state(AppState::InGame)),
        )
        .run()
}

#[derive(States, Debug, Default, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    Boot,
    ModLoading,
    MainMenu,
    InGame,
    Shutdown,
}

fn boot(mut state: ResMut<NextState<AppState>>) {
    state.set(AppState::ModLoading);
}

fn setup(mut commands: Commands, sprites: Res<TileSprites>) {
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
    ));

    for x in -1..=1 {
        for y in -1..=1 {
            let position = Vec2::new(x as f32 * TILE_SIZE as f32, y as f32 * TILE_SIZE as f32);
            commands.spawn((
                Sprite::from_color(Color::hsv(100.0, 1.0, 0.5), Vec2 { x: 16.0, y: 16.0 }),
                WorldTransform::from_xy(position.x, position.y),
            ));
        }
    }
}

#[derive(Component)]
#[require(Transform)]
struct Player;

fn camera_follow(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    camera.single_mut().unwrap().translation = player.single().unwrap().translation;
}

fn player_follow(player: Query<&WorldTransform, With<Player>>, mut base: ResMut<BaseChunk>) {
    let world_transform = player.single().unwrap();
    if world_transform.chunk != base.0 {
        base.0 = world_transform.chunk;
    }
}

fn player_move(
    time: Res<Time>,
    mut player: Query<&mut WorldTransform, With<Player>>,
    input: Res<InputState>,
    registry: Res<Registry<InputAction>>,
) {
    let mut transform = player.single_mut().unwrap();

    let up = registry.lookup("base::up").unwrap();
    let down = registry.lookup("base::down").unwrap();
    let left = registry.lookup("base::left").unwrap();
    let right = registry.lookup("base::right").unwrap();
    let fast = registry.lookup("base::fast").unwrap();

    let axes = input.vec2(right, left, up, down);
    let speed = match input.pressed(fast) {
        false => 8.0,
        true => 32.0,
    };

    *transform += axes.normalize_or_zero() * speed * time.delta_secs();
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
    player: Query<&WorldTransform, With<Player>>,
    input: Res<InputState>,
    registry: Res<Registry<InputAction>>,
) {
    let player_pos = player.single().unwrap().clone().into();
    if input.just_pressed(registry.lookup("base::toggle").unwrap()) {
        let tile = world.get_tile_mut(player_pos).unwrap();
        if tile.id == Id::ZERO {
            tile.id = Id::ONE;
        } else {
            tile.id = Id::ZERO;
        }
    }
}

fn esc_exit(input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
