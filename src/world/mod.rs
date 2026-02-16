use std::collections::HashMap;

use bevy::prelude::*;

use chunk::Chunk;
use tile::Tile;

use crate::world::transform::{apply_rebase, apply_world_transform};

pub use position::WorldPosition;
pub use transform::{BaseChunk, RebaseSet, WorldTransform};

pub mod chunk;
pub mod machine;
mod position;
pub mod tile;
mod transform;

/// Tile size in tiles (n by n)
pub const CHUNK_SIZE: usize = 16;

/// Tile size in pixels (n by n)
pub const TILE_SIZE: usize = 16;

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

    pub fn get_tile(&self, pos: WorldPosition) -> Option<&Tile> {
        let chunk = pos.chunk;
        let tile = pos.tile;
        self.chunks.get(&chunk).map(|chunk| chunk.get(tile))
    }

    pub fn get_tile_mut(&mut self, pos: WorldPosition) -> Option<&mut Tile> {
        let chunk = pos.chunk;
        let tile = pos.tile;
        self.chunks.get_mut(&chunk).map(|chunk| chunk.get_mut(tile))
    }
}
