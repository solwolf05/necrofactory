use std::collections::HashSet;

use bevy::prelude::*;

use crate::{
    GameState,
    math::HybridVec2,
    modding::TileSprites,
    world::{BaseChunk, RebaseSet, TILE_SIZE, World, WorldTransform, chunk::TilePosition},
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
    mut chunks: Query<(&mut RenderChunk, &WorldTransform, &mut Children)>,
    mut tiles: Query<(&RenderTile, &mut Sprite, &mut Visibility)>,
    mut world: ResMut<World>,
    sprites: Res<TileSprites>,
) {
    for (mut render_chunk, transform, children) in chunks.iter_mut() {
        let Some(chunk) = world.get_chunk_mut(transform.translation.chunk()) else {
            continue;
        };

        if !chunk.dirty && !render_chunk.1 {
            continue;
        }

        for child in children.iter() {
            let Ok((render_tile, mut sprite, mut visibility)) = tiles.get_mut(child) else {
                continue;
            };
            let pos = render_tile.0;
            let Some(tile) = chunk.get(pos) else {
                if *visibility != Visibility::Hidden {
                    *visibility = Visibility::Hidden;
                }
                continue;
            };
            let id = tile.id;

            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
            let image = sprites.get(id);
            if sprite.image != image {
                sprite.image = image;
            }
        }

        chunk.dirty = false;
        render_chunk.1 = false;
    }
}

fn spawn_chunks(
    mut commands: Commands,
    base: Res<BaseChunk>,
    render_chunks: Query<(&mut RenderChunk, &mut WorldTransform)>,
    world: Res<World>,
) {
    const CHUNK_RADIUS: i32 = 1;

    if !base.is_changed() {
        return;
    }

    // Calculate which chunks should be loaded
    let mut chunks_to_load = HashSet::new();
    for cy in -CHUNK_RADIUS..=CHUNK_RADIUS {
        for cx in -CHUNK_RADIUS..=CHUNK_RADIUS {
            let chunk_pos = base.0 + IVec2::new(cx, cy);
            chunks_to_load.insert(chunk_pos);
        }
    }

    // Determine which chunks to spawn vs unload
    let mut chunks_in_range = HashSet::new();

    let mut pool = Vec::new();
    for (render_chunk, transform) in render_chunks {
        chunks_in_range.insert(render_chunk.0);

        // If this chunk is no longer in range, despawn it
        if !chunks_to_load.contains(&render_chunk.0) {
            // *visibility = Visibility::Hidden;
            pool.push((render_chunk, transform));
        }
    }

    // Spawn chunks that aren't already loaded
    for chunk_pos in chunks_to_load {
        if chunks_in_range.contains(&chunk_pos) || !world.contains_chunk(chunk_pos) {
            continue;
        };

        if let Some((mut render_chunk, mut transform)) = pool.pop() {
            render_chunk.0 = chunk_pos;
            render_chunk.1 = true;
            transform.translation = HybridVec2::from_chunk(chunk_pos);
            // *visibility = Visibility::Visible;
            continue;
        }

        let mut render_chunk = commands.spawn((
            RenderChunk(chunk_pos, true),
            WorldTransform::from_chunk(chunk_pos),
        ));

        render_chunk.with_children(|spawner| {
            for tile_pos in 0..=255 {
                let tile_pos = TilePosition::new(tile_pos);
                let (x, y) = tile_pos.to_xy();
                let translation = (Vec2::new(x as f32, y as f32) * TILE_SIZE as f32).extend(0.0);
                spawner.spawn((
                    RenderTile(tile_pos),
                    Sprite::default(),
                    Transform::from_translation(translation),
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
#[require(Visibility)]
#[require(Transform)]
pub struct RenderChunk(pub IVec2, pub bool);

#[derive(Debug, Component)]
pub struct RenderTile(pub TilePosition);
