use std::collections::HashMap;

use bevy::prelude::*;

use chunk::{Chunk, TilePosition};
use tile::Tile;

pub mod chunk;
pub mod machine;
pub mod tile;

/// Tile size in tiles (n by n)
pub const CHUNK_SIZE: usize = 16;
/// Tile size in pixels (n by n)
pub const TILE_SIZE: usize = 16;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>();
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPosition {
    pub chunk: IVec2,
    pub tile: TilePosition,
}

impl WorldPosition {
    pub fn new(chunk: IVec2, tile: TilePosition) -> Self {
        Self { chunk, tile }
    }

    /// Panics if out of bounds, which hopefully isnt a problem unless the player goes really far away
    pub fn from_xy(x: i64, y: i64) -> Self {
        let cs = CHUNK_SIZE as i64;

        let chunk_x = x.div_euclid(cs);
        let chunk_y = y.div_euclid(cs);

        let tile_x = x.rem_euclid(cs);
        let tile_y = y.rem_euclid(cs);

        Self::new(
            IVec2::new(chunk_x.try_into().unwrap(), chunk_y.try_into().unwrap()),
            TilePosition::from_xy(tile_x as u8, tile_y as u8).unwrap(),
        )
    }

    pub fn from_bevy(pos: Vec3) -> WorldPosition {
        let pos = pos.truncate();

        // 1. world space → global tile coordinate
        let tx = (pos.x / TILE_SIZE as f32).floor() as i64;
        let ty = (pos.y / TILE_SIZE as f32).floor() as i64;

        Self::from_xy(tx, ty)
    }

    pub fn to_bevy(self) -> Vec3 {
        todo!()
    }

    pub fn x(&self) -> i64 {
        self.chunk.x as i64 * CHUNK_SIZE as i64 + self.tile.x() as i64
    }

    pub fn y(&self) -> i64 {
        self.chunk.y as i64 * CHUNK_SIZE as i64 + self.tile.y() as i64
    }

    pub fn to_xy(&self) -> (i64, i64) {
        (self.x(), self.y())
    }
}
