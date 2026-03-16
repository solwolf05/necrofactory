use bevy::prelude::*;
use noiz::NoiseFunction;

use crate::{
    AppState,
    modding::Id,
    rand::{MasterSeed, WorldHeightRng},
    world::{
        BaseChunk, CHUNK_SIZE, World,
        chunk::{Chunk, TilePosition},
        tile::Tile,
    },
};

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dynamic_gen.run_if(in_state(AppState::InGame)));
    }
}

pub fn dev_gen(world: ResMut<World>) {
    let world = world.into_inner();
    for x in -1000..=1000 {
        for y in -1000..=1000 {
            let pos = IVec2::new(x, y);
            // test_gen_chunk(world, pos, 0);
        }
    }
}

pub fn dynamic_gen(world: ResMut<World>, base: Res<BaseChunk>, mut seed: ResMut<WorldHeightRng>) {
    let world = world.into_inner();
    for cy in -8..=8 {
        for cx in -8..=8 {
            let chunk_pos = base.0 + IVec2::new(cx, cy);
            if !world.contains_chunk(chunk_pos) {
                test_gen_chunk(world, chunk_pos, &mut *seed);
            }
        }
    }
}

pub fn test_gen_chunk(world: &mut World, pos: IVec2, rng: &mut WorldHeightRng) {
    let mut chunk = Chunk::empty();
    let chunk_factor = rand::random::<f32>() / 2.0 + 0.5;
    if pos.y < -1 {
        for tile in chunk.iter_mut() {
            *tile = Tile { id: Id::ONE };
        }
    } else if pos.y == -1 {
        for x in 0..=15 {
            let height = rng
                .1
                .evaluate(i32::cast_unsigned((pos.x * 16) + x as i32), &mut rng.0)
                * 16.0;
            info!("{}", height);
            for y in 0..height.floor() as u8 {
                chunk.insert(TilePosition::from_xy(x, y), Tile::new(Id::ONE));
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
