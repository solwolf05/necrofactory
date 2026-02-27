use bevy::prelude::*;

use crate::{
    AppState,
    modding::Id,
    world::{BaseChunk, World, chunk::Chunk, tile::Tile},
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
            test_gen_chunk(world, pos);
        }
    }
}

pub fn dynamic_gen(world: ResMut<World>, base: Res<BaseChunk>) {
    let world = world.into_inner();
    for cy in -8..=8 {
        for cx in -8..=8 {
            let chunk_pos = base.0 + IVec2::new(cx, cy);
            if !world.contains_chunk(chunk_pos) {
                test_gen_chunk(world, chunk_pos);
            }
        }
    }
}

pub fn test_gen_chunk(world: &mut World, pos: IVec2) {
    let mut chunk = Chunk::empty();
    for tile in chunk.iter_mut() {
        let factor = pos.length_squared() as f32 / 100.0 / 100.0;
        if rand::random::<f32>() < factor {
            *tile = Tile { id: Id::ONE };
        }
    }
    world.insert_chunk(pos, chunk);
}
