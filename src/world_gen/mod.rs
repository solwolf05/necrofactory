use bevy::prelude::*;

use crate::{
    GameState,
    modding::{Id, Registry},
    world::{
        BaseChunk, World,
        chunk::{Chunk, TilePosition},
        tile::{Tile, TileDef},
    },
};

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dynamic_gen.run_if(in_state(GameState::InGame)));
    }
}

pub fn dynamic_gen(world: ResMut<World>, base: Res<BaseChunk>, registry: Res<Registry<TileDef>>) {
    let world = world.into_inner();
    let registry = registry.into_inner();
    for cy in -8..=8 {
        for cx in -8..=8 {
            let chunk_pos = base.0 + IVec2::new(cx, cy);
            if !world.contains_chunk(chunk_pos) {
                test_gen_chunk(world, chunk_pos, registry);
            }
        }
    }
}

pub fn test_gen_chunk(world: &mut World, pos: IVec2, registry: &Registry<TileDef>) {
    let mut chunk = Chunk::empty();
    let chunk_factor = rand::random::<f32>() / 2.0 + 0.5;
    if pos.y < -1 {
        for tile in chunk.iter_mut() {
            *tile = Tile { id: Id::ONE };
        }
    } else if pos.y == -1 {
        let random = rand::random_range(1..registry.len());
        let tile = Tile::new(Id::new(random as u32));
        for x in 0..=15 {
            for y in 0..=15 as u8 {
                chunk.insert(TilePosition::from_xy(x, y), tile);
            }
        }
    } else {
        for tile in chunk.iter_mut() {
            let radius = 10.0;
            let factor = pos.y as f32 / radius / radius * chunk_factor;
            if rand::random::<f32>() < factor {
                *tile = Tile { id: Id::ONE };
            }
        }
    }
    world.insert_chunk(pos, chunk);
}
