use std::collections::HashMap;

use bevy::{math::I64Vec2, prelude::*};

use chunk::Chunk;
use tile::Tile;

use crate::{
    GameState,
    world::{
        chunk::TilePosition,
        transform::{apply_rebase, apply_world_transform},
    },
};

pub use transform::{BaseChunk, RebaseSet, WorldTransform};

pub mod chunk;
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
        app.add_systems(OnEnter(GameState::InGame), setup)
            .add_systems(
                PostUpdate,
                (apply_rebase, apply_world_transform)
                    .in_set(RebaseSet)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(OnExit(GameState::InGame), cleanup);
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<World>();
    commands.init_resource::<BaseChunk>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<World>();
    commands.remove_resource::<BaseChunk>();
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

    pub fn insert_chunk(&mut self, pos: IVec2, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    pub fn remove_chunk(&mut self, pos: IVec2) {
        self.chunks.remove(&pos);
    }

    pub fn contains_chunk(&self, pos: IVec2) -> bool {
        self.chunks.contains_key(&pos)
    }

    pub fn get_chunk(&self, pos: IVec2) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_chunk_mut(&mut self, pos: IVec2) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
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

    pub fn remove_tile(&mut self, pos: I64Vec2) {
        let chunk_pos = pos.div_euclid(I64Vec2::splat(CHUNK_SIZE as i64)).as_ivec2();
        let tile_pos = pos.rem_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.remove(TilePosition::from_xy(tile_pos.x as u8, tile_pos.y as u8));
        }
    }

    pub fn contains_tile(&self, pos: I64Vec2) -> bool {
        self.get_tile(pos).is_some()
    }

    pub fn get_tile(&self, pos: I64Vec2) -> Option<&Tile> {
        let chunk_pos = pos.div_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        let tile_pos = pos.rem_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        self.chunks
            .get(&chunk_pos.as_ivec2())
            .and_then(|chunk| chunk.get(TilePosition::from_xy(tile_pos.x as u8, tile_pos.y as u8)))
    }

    pub fn get_tile_mut(&mut self, pos: I64Vec2) -> Option<&mut Tile> {
        let chunk_pos = pos.div_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        let tile_pos = pos.rem_euclid(I64Vec2::splat(CHUNK_SIZE as i64));
        self.chunks
            .get_mut(&chunk_pos.as_ivec2())
            .and_then(|chunk| {
                chunk.get_mut(TilePosition::from_xy(tile_pos.x as u8, tile_pos.y as u8))
            })
    }
}
