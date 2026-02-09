use std::collections::HashMap;

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashSet,
    prelude::*,
};
use bevy_modding::prelude::*;

use crate::{
    Player,
    modding::{TileTextures, all_tile_textures_loaded},
    world::{
        CHUNK_SIZE, TILE_SIZE, World, WorldPosition,
        chunk::{Chunk, TilePosition},
        tile::TileDef,
    },
};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostModLoad,
            build_texture_atlas_system.after(all_tile_textures_loaded),
        )
        .add_systems(Update, spawn_test_chunks)
        .add_systems(PostUpdate, build_meshes);
    }
}

#[derive(Debug, Resource)]
struct TextureAtlasCache {
    pub map: HashMap<Id<TileDef>, u32>,
    pub layout: TextureAtlasLayout,
    pub sources: TextureAtlasSources,
    pub texture: Handle<Image>,
}

fn build_texture_atlas_system(
    mut commands: Commands,
    tile_textures: Res<TileTextures>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut builder = TextureAtlasBuilder::default();
    builder.padding(UVec2::splat(2));

    let mut map = HashMap::new();

    info!("{:?}", tile_textures);

    for (_tile_id, handle) in &tile_textures.handles {
        let image = images.get(handle).unwrap();
        builder.add_texture(Some(handle.id()), image);
    }

    let (layout, sources, mut atlas_image) = builder.build().unwrap();

    // pixel art
    atlas_image.sampler = ImageSampler::nearest();

    let texture = images.add(atlas_image);

    // map tile → atlas rect index
    for (&source, &rect_index) in &sources.texture_ids {
        let tile = tile_textures
            .handles
            .iter()
            .find(|(_, h)| h.id() == source)
            .unwrap();

        map.insert(tile.0.clone(), rect_index as u32);
    }

    commands.insert_resource(TextureAtlasCache {
        map,
        layout,
        sources,
        texture,
    });
}

fn atlas_uv(rect: &URect, atlas_size: UVec2) -> [[f32; 2]; 4] {
    let w = atlas_size.x as f32;
    let h = atlas_size.y as f32;

    let u0 = rect.min.x as f32 / w;
    let v0 = rect.min.y as f32 / h;
    let u1 = rect.max.x as f32 / w;
    let v1 = rect.max.y as f32 / h;

    [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]
}

fn build_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&RenderChunk, &mut Mesh2d)>,
    mut world: ResMut<World>,
    atlas: Res<TextureAtlasCache>,
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

fn build_chunk_mesh(chunk: &Chunk, atlas: &TextureAtlasCache) -> Mesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let mut index_offset = 0;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get(TilePosition::from_xy(x as u8, y as u8).unwrap());
            if tile.id.0 == 0 {
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

            info!("{:?}", atlas);

            let atlas_index = atlas.map[&tile.id];
            let rect = atlas.layout.textures[atlas_index as usize];
            let tile_uvs = atlas_uv(&rect, atlas.layout.size);

            // TEMP UVs (replace with atlas math later)
            uvs.extend_from_slice(&tile_uvs);

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

fn spawn_test_chunks(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    render_chunks: Query<(Entity, &RenderChunk)>,
    world: Res<World>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Get player position
    let player_transform = player.single().unwrap();
    let player_chunk = WorldPosition::from_bevy(player_transform.translation).chunk;

    // Calculate which chunks should be loaded (3x3 area)
    let mut chunks_to_load = Vec::new();
    for cy in -1..=1 {
        for cx in -1..=1 {
            let chunk_pos = player_chunk + IVec2::new(cx, cy);
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

            let world_pos = Vec3::new(
                chunk_pos.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE as f32,
                chunk_pos.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE as f32,
                0.0,
            );

            commands.spawn((
                RenderChunk(chunk_pos, true),
                Mesh2d::default(),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::hsl(
                    rand::random::<f32>() * 360.0,
                    1.0,
                    0.5,
                )))),
                Transform::from_translation(world_pos),
            ));
        }
    }
}

#[derive(Component)]
#[require(Mesh2d)]
#[require(MeshMaterial2d<ColorMaterial>)]
pub struct RenderChunk(pub IVec2, pub bool);
