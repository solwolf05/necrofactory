use bevy::prelude::*;

use crate::{AppState, Player, world::WorldPosition};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(Update, update_text.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct WorldPosText;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ChunkPosText;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct TilePosText;

fn setup(mut commands: Commands) {
    let text_root = commands
        .spawn((
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            },
        ))
        .id();

    commands.entity(text_root).with_children(|parent| {
        parent.spawn((TextSpan::new("World"), WorldPosText));
        parent.spawn((TextSpan::new("Chunk"), ChunkPosText));
        parent.spawn((TextSpan::new("Tile"), TilePosText));
    });
}

fn update_text(
    player: Query<&Transform, With<Player>>,
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

    let world_pos = WorldPosition::from_bevy(player.translation);
    let chunk_pos = world_pos.chunk;
    let tile_pos = world_pos.tile;

    world.0 = format!("World: {}\n", world_pos);
    chunk.0 = format!("Chunk: {}\n", chunk_pos);
    tile.0 = format!("Tile: {}\n", tile_pos);
}
