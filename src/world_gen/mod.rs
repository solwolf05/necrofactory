use bevy_modding::prelude::*;

use crate::world::{
    ChunkPosition, World,
    chunk::{Chunk, TilePosition},
    tile::Tile,
};

pub fn dev_gen(world: &mut World) {
    for x in -16..=16 {
        for y in -16..=16 {
            let pos = ChunkPosition::new(x, y);
            let mut chunk = Chunk::empty();
            if pos.y < 0 {
                for i in 0..=255 {
                    chunk.insert(TilePosition::new(i), Tile::new(RegHandle::new(0)));
                }
            }
            world.insert_chunk(pos, chunk);
        }
    }
}
