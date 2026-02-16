use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use bevy::{math::I64Vec2, prelude::*};

use crate::world::{CHUNK_SIZE, chunk::TilePosition, transform::WorldTransform};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldPosition {
    pub chunk: IVec2,
    pub tile: TilePosition,
}

impl WorldPosition {
    pub fn new(chunk: IVec2, tile: TilePosition) -> Self {
        Self { chunk, tile }
    }

    pub fn from_chunk(chunk: IVec2) -> Self {
        Self::new(chunk, TilePosition::ZERO)
    }

    pub fn from_tile(tile: TilePosition) -> Self {
        Self::new(IVec2::ZERO, tile)
    }

    /// Panics if out of bounds, which hopefully isnt a problem unless the player goes really far away
    pub fn from_xy(x: i64, y: i64) -> Self {
        let cs = CHUNK_SIZE as i64;

        let chunk_x = x.div_euclid(cs) as i32;
        let chunk_y = y.div_euclid(cs) as i32;

        let tile_x = x.rem_euclid(cs) as u8;
        let tile_y = y.rem_euclid(cs) as u8;

        Self::new(
            IVec2::new(chunk_x, chunk_y),
            TilePosition::from_xy(tile_x, tile_y),
        )
    }

    pub fn x(&self) -> i64 {
        self.chunk.x as i64 * CHUNK_SIZE as i64 + self.tile.x() as i64
    }

    pub fn y(&self) -> i64 {
        self.chunk.y as i64 * CHUNK_SIZE as i64 + self.tile.y() as i64
    }

    pub fn to_i64vec2(&self) -> I64Vec2 {
        I64Vec2::new(self.x(), self.y())
    }
}

impl Display for WorldPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x(), self.y())
    }
}

impl From<WorldTransform> for WorldPosition {
    fn from(value: WorldTransform) -> Self {
        Self {
            chunk: value.chunk,
            tile: TilePosition::from_xy(value.tile.x.floor() as u8, value.tile.y.floor() as u8),
        }
    }
}

impl Add for WorldPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let chunk_size = CHUNK_SIZE as i64;
        let cx = self.chunk.x as i64 + rhs.chunk.x as i64;
        let cy = self.chunk.y as i64 + rhs.chunk.y as i64;

        let tx = self.tile.x() as i64 + rhs.tile.x() as i64;
        let ty = self.tile.y() as i64 + rhs.tile.y() as i64;

        // Handle underflow for x
        let (tx, cx) = if tx >= chunk_size {
            (tx - chunk_size, cx + 1)
        } else if tx < 0 {
            (tx + chunk_size, cx - 1)
        } else {
            (tx, cx)
        };

        // Handle underflow for y
        let (ty, cy) = if ty >= chunk_size {
            (ty - chunk_size, cy + 1)
        } else if ty < 0 {
            (ty + chunk_size, cy - 1)
        } else {
            (ty, cy)
        };

        WorldPosition::new(
            IVec2::new(cx as i32, cy as i32),
            TilePosition::from_xy(tx as u8, ty as u8),
        )
    }
}

impl Sub for WorldPosition {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let chunk_size = CHUNK_SIZE as i64;
        let cx = self.chunk.x as i64 - rhs.chunk.x as i64;
        let cy = self.chunk.y as i64 - rhs.chunk.y as i64;

        let tx = self.tile.x() as i64 - rhs.tile.x() as i64;
        let ty = self.tile.y() as i64 - rhs.tile.y() as i64;

        // Handle underflow for x
        let (tx, cx) = if tx >= chunk_size {
            (tx - chunk_size, cx + 1)
        } else if tx < 0 {
            (tx + chunk_size, cx - 1)
        } else {
            (tx, cx)
        };

        // Handle underflow for y
        let (ty, cy) = if ty >= chunk_size {
            (ty - chunk_size, cy + 1)
        } else if ty < 0 {
            (ty + chunk_size, cy - 1)
        } else {
            (ty, cy)
        };

        WorldPosition::new(
            IVec2::new(cx as i32, cy as i32),
            TilePosition::from_xy(tx as u8, ty as u8),
        )
    }
}

impl AddAssign for WorldPosition {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for WorldPosition {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
