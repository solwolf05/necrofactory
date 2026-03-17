use crate::{debug::DebugText, player::Player, world::WorldTransform};
use bevy::prelude::*;

use crate::GameState;

pub struct CoordinatePlugin;

impl Plugin for CoordinatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup.after(super::setup))
            .add_systems(Update, update_text.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct WorldPosText;

#[derive(Component)]
pub struct ChunkPosText;

#[derive(Component)]
pub struct TilePosText;

fn setup(mut commands: Commands, text_root: Query<Entity, With<DebugText>>) {
    let text_root = text_root.single().unwrap();
    commands.entity(text_root).with_children(|parent| {
        parent.spawn((TextSpan::new("World"), WorldPosText));
        parent.spawn((TextSpan::new("Chunk"), ChunkPosText));
        parent.spawn((TextSpan::new("Tile"), TilePosText));
    });
}

fn update_text(
    player: Query<&WorldTransform, With<Player>>,
    mut world: Query<&mut TextSpan, With<WorldPosText>>,
    mut chunk: Query<&mut TextSpan, (With<ChunkPosText>, Without<WorldPosText>)>,
    mut tile: Query<
        &mut TextSpan,
        (
            With<TilePosText>,
            Without<ChunkPosText>,
            Without<WorldPosText>,
        ),
    >,
) {
    let player = player.single().unwrap();
    let mut world = world.single_mut().unwrap();
    let mut chunk = chunk.single_mut().unwrap();
    let mut tile = tile.single_mut().unwrap();

    let world_pos = player.translation;
    let chunk_pos = world_pos.chunk();
    let tile_pos = world_pos.tile();

    world.0 = format!("World: {:.2}\n", world_pos);
    chunk.0 = format!("Chunk: {}\n", chunk_pos);
    tile.0 = format!("Tile: {:.2}\n", tile_pos);
}
