use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    GameState,
    math::HybridVec2,
    modding::{Id, TileSprites},
    world::{BaseChunk, RebaseSet, World, WorldTransform, chunk::TilePosition},
};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (spawn_chunks.before(RebaseSet), update_sprites)
                .chain()
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(OnExit(GameState::InGame), cleanup);
    }
}

fn update_sprites(
    mut chunks: Query<(&mut RenderChunk, &mut Children)>,
    mut tiles: Query<(&RenderTile, &mut Sprite, &mut Visibility)>,
    mut world: ResMut<World>,
    sprites: Res<TileSprites>,
) {
    for (mut render_chunk, children) in chunks.iter_mut() {
        let Some(chunk) = world.get_chunk_mut(render_chunk.0) else {
            continue;
        };

        if !chunk.dirty && !render_chunk.1 {
            continue;
        }

        for child in children.iter() {
            if let Ok((render_tile, mut sprite, mut visibility)) = tiles.get_mut(child) {
                let pos = render_tile.0;
                let tile = chunk.get(pos);
                let id = tile.id;

                if id == Id::ZERO {
                    *visibility = Visibility::Hidden;
                } else {
                    *visibility = Visibility::Visible;
                    sprite.image = sprites.get(id);
                }
            }
        }

        chunk.dirty = false;
        render_chunk.1 = false;
    }
}

fn spawn_chunks(
    mut commands: Commands,
    base: Res<BaseChunk>,
    render_chunks: Query<(Entity, &RenderChunk)>,
    world: Res<World>,
) {
    const CHUNK_RADIUS: i32 = 1;

    // Calculate which chunks should be loaded (3x3 area)
    let mut chunks_to_load = Vec::new();
    for cy in -CHUNK_RADIUS..=CHUNK_RADIUS {
        for cx in -CHUNK_RADIUS..=CHUNK_RADIUS {
            let chunk_pos = base.0 + IVec2::new(cx, cy);
            chunks_to_load.push(chunk_pos);
        }
    }

    // Determine which chunks to spawn vs unload
    let mut chunks_in_range = HashSet::new();

    for (entity, render_chunk) in render_chunks {
        chunks_in_range.insert(render_chunk.0);

        // If this chunk is no longer in range, despawn it
        if !chunks_to_load.contains(&render_chunk.0) {
            commands.entity(entity).despawn();
        }
    }

    // Spawn chunks that aren't already loaded
    for chunk_pos in chunks_to_load {
        if chunks_in_range.contains(&chunk_pos) || !world.contains_chunk(chunk_pos) {
            continue;
        };

        let mut render_chunk = commands.spawn((RenderChunk(chunk_pos, true), Transform::default()));

        render_chunk.with_children(|spawner| {
            for tile_pos in 0..=255 {
                let tile_pos = TilePosition::new(tile_pos);
                let (x, y) = tile_pos.to_xy();
                let tile_vec2 = Vec2::new(x as f32, y as f32);
                let translation = HybridVec2::from_chunk_tile(chunk_pos, tile_vec2);
                spawner.spawn((
                    RenderTile(tile_pos),
                    Sprite::default(),
                    WorldTransform::from_translation(translation),
                ));
            }
        });
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<RenderChunk>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

#[derive(Debug, Component)]
pub struct RenderTile(pub TilePosition);

#[derive(Debug, Component)]
#[require(InheritedVisibility)]
pub struct RenderChunk(pub IVec2, pub bool);
