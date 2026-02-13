use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashSet,
    prelude::*,
};

use crate::{
    AppState, Player,
    modding::types::Id,
    world::{
        CHUNK_SIZE, TILE_SIZE, World, WorldPosition,
        chunk::{Chunk, TilePosition},
        tile::TileDef,
    },
};

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_test_chunks.run_if(in_state(AppState::InGame)))
            .add_systems(PostUpdate, build_meshes.run_if(in_state(AppState::InGame)));
    }
}

// fn build_atlas_system(
//     mut commands: Commands,
//     mut registry: ResMut<Registry<TileDef>>,
//     mut asset_server: Res<AssetServer>,
//     mut images: ResMut<Assets<Image>>,
//     mut layouts: ResMut<Assets<TextureAtlasLayout>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     // 1) Create a TextureAtlasBuilder and add all registered images
//     let mut builder = TextureAtlasBuilder::default();

//     let mut handles = Vec::new();
//     for (_, tile) in registry.iter() {
//         handles.push(asset_server.load(tile.sprite_path));
//     }
//     for handle in handles {
//         let image = images.get(&handle).unwrap();
//         builder.add_texture(None, image);
//     }

//     // 2) build atlas (packer does the work). returns (layout, sources, image)
//     let (atlas_layout, _sources, atlas_image) = builder.build().expect("atlas build failed");

//     // 3) add image & layout into asset storage
//     let atlas_image_handle = images.add(atlas_image);
//     let atlas_layout_handle = layouts.add(atlas_layout);

//     // 4) create a ColorMaterial that uses the atlas image
//     let material_handle = materials.add(ColorMaterial::from(atlas_image_handle.clone()));

//     commands.insert_resource(Atlas {
//         image_handle: atlas_image_handle,
//         layout_handle: atlas_layout_handle,
//         material_handle,
//     });

//     // registry can be dropped or kept for metadata mapping (tile id -> index)
// }

fn build_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&RenderChunk, &mut Mesh2d)>,
    mut world: ResMut<World>,
) {
    for (render_chunk, mut mesh2d) in query.iter_mut() {
        let Some(chunk) = world.get_chunk_mut(render_chunk.0) else {
            continue;
        };

        if !chunk.dirty && !render_chunk.1 {
            continue;
        }

        let mesh = build_chunk_mesh(chunk);
        *mesh2d = Mesh2d(meshes.add(mesh));

        chunk.dirty = false;
    }
}

fn build_chunk_mesh(chunk: &Chunk) -> Mesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let mut index_offset = 0;

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            let tile = chunk.get(TilePosition::from_xy(x as u8, y as u8).unwrap());
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
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

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
