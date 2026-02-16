use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashSet,
    prelude::*,
    sprite_render::Material2dPlugin,
};

use crate::{
    AppState, Player,
    graphics::atlas::{TextureAtlasMap, TilemapMaterial, build_texture_atlas},
    modding::{Id, ModLoadState, Registry},
    world::{
        BaseChunk, CHUNK_SIZE, RebaseSet, TILE_SIZE, World, WorldPosition, WorldTransform,
        chunk::{Chunk, TilePosition},
        tile::TileDef,
    },
};

pub mod atlas;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TilemapMaterial>::default())
            .add_systems(OnEnter(ModLoadState::Finalize), build_texture_atlas)
            .add_systems(
                PostUpdate,
                (spawn_chunks.before(RebaseSet), build_meshes)
                    .chain()
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn build_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&RenderChunk, &mut Mesh2d)>,
    mut world: ResMut<World>,
    atlas: Res<TextureAtlasMap>,
) {
    let atlas = atlas.into_inner();
    for (render_chunk, mut mesh2d) in query.iter_mut() {
        let Some(chunk) = world.get_chunk_mut(render_chunk.0) else {
            continue;
        };

        if !chunk.dirty && !render_chunk.1 {
            continue;
        }

        let mesh = build_chunk_mesh(chunk, atlas);
        *mesh2d = Mesh2d(meshes.add(mesh));

        chunk.dirty = false;
    }
}

fn build_chunk_mesh(chunk: &Chunk, atlas: &TextureAtlasMap) -> Mesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let mut index_offset = 0;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get(TilePosition::from_xy(x as u8, y as u8));
            if tile.id == Id::ZERO {
                continue;
            }

            let x = x as f32 * TILE_SIZE as f32;
            let y = y as f32 * TILE_SIZE as f32;

            // Quad vertices
            positions.extend_from_slice(&[
                [x, y, 0.0],
                [x + TILE_SIZE as f32, y, 0.0],
                [x + TILE_SIZE as f32, y + TILE_SIZE as f32, 0.0],
                [x, y + TILE_SIZE as f32, 0.0],
            ]);

            // TEMP UVs (replace with atlas math later)
            // uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

            let rect = match atlas.rect(tile.id) {
                Some(r) => r,
                None => continue,
            };

            let min = rect.min;
            let max = rect.max;

            // IMPORTANT: Bevy UV origin is bottom-left
            uvs.extend_from_slice(&[
                [min.x, max.y],
                [max.x, max.y],
                [max.x, min.y],
                [min.x, min.y],
            ]);

            indices.extend_from_slice(&[
                index_offset,
                index_offset + 1,
                index_offset + 2,
                index_offset,
                index_offset + 2,
                index_offset + 3,
            ]);

            index_offset += 4;
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

fn spawn_chunks(
    mut commands: Commands,
    base: Res<BaseChunk>,
    render_chunks: Query<(Entity, &RenderChunk)>,
    world: Res<World>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    atlas: Res<TextureAtlasMap>,
) {
    // Calculate which chunks should be loaded (3x3 area)
    let mut chunks_to_load = Vec::new();
    for cy in -1..=1 {
        for cx in -1..=1 {
            let chunk_pos = base.0 + IVec2::new(cx, cy);
            chunks_to_load.push(chunk_pos);
        }
    }

    // Determine which chunks to spawn vs unload
    let mut chunks_in_range = HashSet::new();

    for (entity, render_chunk) in &render_chunks {
        chunks_in_range.insert(render_chunk.0);

        // If this chunk is no longer in range, despawn it
        if !chunks_to_load.contains(&render_chunk.0) {
            commands.entity(entity).despawn();
        }
    }

    // Spawn chunks that aren't already loaded
    for chunk_pos in chunks_to_load {
        if !chunks_in_range.contains(&chunk_pos) {
            let Some(chunk) = world.get_chunk(chunk_pos) else {
                continue;
            };

            commands.spawn((
                RenderChunk(chunk_pos, true),
                Mesh2d::default(),
                MeshMaterial2d(atlas.material.clone()),
                WorldTransform::from_chunk(chunk_pos),
            ));
        }
    }
}

#[derive(Component)]
#[require(Mesh2d)]
#[require(MeshMaterial2d<TilemapMaterial>)]
pub struct RenderChunk(pub IVec2, pub bool);
