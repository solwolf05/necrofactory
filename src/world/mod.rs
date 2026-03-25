use std::collections::HashMap;

use bevy::{math::I64Vec2, prelude::*};

use chunk::Chunk;
use tile::Tile;

use crate::world::{
    chunk::TilePosition,
    transform::{apply_rebase, apply_world_transform},
};

pub use transform::{BaseChunk, RebaseSet, WorldTransform};

pub mod chunk;
pub mod machine;
mod position;
pub mod tile;
mod transform;

/// Tile size in tiles (n by n)
pub const CHUNK_SIZE: i32 = 16;

/// Tile size in pixels (n by n)
pub const TILE_SIZE: i32 = 16;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .init_resource::<BaseChunk>()
            .add_systems(
                PostUpdate,
                (apply_rebase, apply_world_transform).in_set(RebaseSet),
            );
    }
}

#[derive(Debug, Default, Resource)]
pub struct World {
    chunks: HashMap<IVec2, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn get_chunk(&self, pos: IVec2) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: IVec2) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn contains_chunk(&self, pos: IVec2) -> bool {
        self.chunks.contains_key(&pos)
    }

    pub fn insert_chunk(&mut self, pos: IVec2, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    pub fn get_tile(&self, pos: I64Vec2) -> Option<&Tile> {
        let chunk_pos = pos.div_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        let tile_pos = pos.rem_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        self.chunks
            .get(&chunk_pos.as_ivec2())
            .map(|chunk| chunk.get(TilePosition::from_xy(tile_pos.x as u8, tile_pos.y as u8)))
    }

    pub fn get_tile_mut(&mut self, pos: I64Vec2) -> Option<&mut Tile> {
        let chunk_pos = pos.div_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        let tile_pos = pos.rem_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        self.chunks
            .get_mut(&chunk_pos.as_ivec2())
            .map(|chunk| chunk.get_mut(TilePosition::from_xy(tile_pos.x as u8, tile_pos.y as u8)))
    }

    pub fn contains_tile(&self, pos: I64Vec2) -> bool {
        self.get_tile(pos).filter(|tile| tile.is_some()).is_some()
    }

    pub fn insert_tile(&mut self, pos: I64Vec2, tile: Tile) {
        let chunk_pos = pos.div_euclid(I64Vec2::splat(CHUNK_SIZE as i64)).as_ivec2();
        let tile_pos = pos.rem_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.insert(
                TilePosition::from_xy(tile_pos.x as u8, tile_pos.y as u8),
                tile,
            );
        } else {
            let mut chunk = Chunk::empty();
            chunk.insert(
                TilePosition::from_xy(tile_pos.x as u8, tile_pos.y as u8),
                tile,
            );
            self.chunks.insert(chunk_pos, chunk);
        }
    }
}
