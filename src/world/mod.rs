use std::collections::HashMap;

use bevy::prelude::*;

use crate::world::{
    chunk::{Chunk, TilePosition},
    tile::Tile,
};

pub mod chunk;
pub mod machine;
pub mod tile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>();
    }
}

#[derive(Debug, Default, Resource)]
pub struct World {
    chunks: HashMap<ChunkPosition, Chunk>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn insert_chunk(&mut self, pos: ChunkPosition, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    pub fn get_tile(&self, pos: WorldPosition) -> Option<&Tile> {
        let chunk = pos.chunk();
        let tile = pos.tile();
        self.chunks.get(&chunk)?.get(tile)
    }
}

/// A chunks position in the world
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldPosition {
    pub chunk: ChunkPosition,
    pub tile: TilePosition,
}

impl WorldPosition {
    pub fn new(chunk: ChunkPosition, tile: TilePosition) -> Self {
        Self { chunk, tile }
    }

    pub fn from_xy(x: i64, y: i64) -> Option<Self> {
        let chunk_x = x / 16;
        let chunk_y = y / 16;
        let tile_x = x % 16;
        let tile_y = y % 16;
        Some(Self::new(
            ChunkPosition::new(chunk_x.try_into().ok()?, chunk_y.try_into().ok()?),
            TilePosition::from_xy(tile_x as u8, tile_y as u8).unwrap(),
        ))
    }

    pub fn chunk(&self) -> ChunkPosition {
        self.chunk
    }

    pub fn tile(&self) -> TilePosition {
        self.tile
    }

    pub fn x(&self) -> i64 {
        self.chunk.x as i64 * 16 + self.tile.x() as i64
    }

    pub fn y(&self) -> i64 {
        self.chunk.y as i64 * 16 + self.tile.y() as i64
    }

    pub fn to_xy(&self) -> (i64, i64) {
        (self.x(), self.y())
    }
}
