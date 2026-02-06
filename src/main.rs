use bevy::{camera::ScalingMode, prelude::*, window::WindowResolution};
use bevy_modding::prelude::*;
use bevy_modding_input::prelude::*;

use crate::{
    modding::ModLoadPlugin,
    world::{World, WorldPlugin},
    world_gen::dev_gen,
};

mod modding;
mod render;
mod serialization;
mod world;
mod world_gen;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            ModLoadPlugin,
            ModPlugin,
            InputPlugin,
            WorldPlugin,
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
        ))
        .add_systems(Startup, (setup, gen_world, asset_test))
        .add_systems(Update, (esc_exit, camera_follow))
        .add_systems(FixedUpdate, player_move)
        .run()
}

fn gen_world(world: ResMut<World>) {
    dev_gen(world.into_inner());
}

fn asset_test(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Sprite::from_image(
        asset_server.load("mods://base/assets/red.png"),
    ));
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

    commands.spawn((Player, Transform::default()));
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

fn player_move(mut player: Query<&mut Transform, With<Player>>, input: Res<InputState>) {
    let mut transform = player.single_mut().unwrap();

    let up = RegHandle::new(0);
    let down = RegHandle::new(1);
    let left = RegHandle::new(2);
    let right = RegHandle::new(3);
    let fast = RegHandle::new(4);

    let axes = input.vec2(right, left, up, down);
    let speed = match input.pressed(fast) {
        true => 4.0,
        false => 1.0,
    };

    transform.translation += axes.normalize_or_zero().extend(0.0) * speed;
}

fn esc_exit(input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
