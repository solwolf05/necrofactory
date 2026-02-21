use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use bevy::prelude::*;

use crate::world::{CHUNK_SIZE, TILE_SIZE, WorldPosition};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RebaseSet;

/// The chunk position that the world is centered around
#[derive(Debug, Default, Resource)]
pub struct BaseChunk(pub IVec2);

/// Used for rendering and other operations that require floating-point coordinates.
/// Separates the world into chunks and tiles to avoid floating point precision issues far from origin.
#[derive(Debug, Default, Clone, Copy, Component)]
#[require(Transform)]
pub struct WorldTransform {
    pub chunk: IVec2,
    pub tile: Vec2,
}

impl WorldTransform {
    pub fn new(chunk: IVec2, tile: Vec2) -> Self {
        Self { chunk, tile }.normalize()
    }

    pub fn from_chunk(chunk: IVec2) -> Self {
        Self::new(chunk, Vec2::ZERO)
    }

    pub fn from_tile(tile: Vec2) -> Self {
        Self::new(IVec2::ZERO, tile)
    }

    /// Creates a WorldTransform from absolute world coordinates.
    pub fn from_xy(x: f32, y: f32) -> Self {
        let cs = CHUNK_SIZE as f32;
        let chunk_x = x.div_euclid(cs) as i32;
        let chunk_y = y.div_euclid(cs) as i32;
        let tile_x = x.rem_euclid(cs);
        let tile_y = y.rem_euclid(cs);
        Self::new(IVec2::new(chunk_x, chunk_y), Vec2::new(tile_x, tile_y))
    }

    pub fn x(&self) -> f32 {
        (self.chunk.x * CHUNK_SIZE as i32) as f32 + self.tile.x
    }

    pub fn y(&self) -> f32 {
        (self.chunk.y * CHUNK_SIZE as i32) as f32 + self.tile.y
    }

    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x(), self.y())
    }

    /// Normalize tile coordinates so they are always within [0, CHUNK_SIZE) and adjust chunk accordingly.
    fn normalize(self) -> Self {
        let cs = CHUNK_SIZE as f32;

        let mut chunk = self.chunk;
        let mut tile = self.tile;

        if tile.x >= cs {
            chunk.x += (tile.x / cs).floor() as i32;
            tile.x = tile.x.rem_euclid(cs);
        } else if tile.x < 0.0 {
            chunk.x -= ((-tile.x) / cs).ceil() as i32;
            tile.x = tile.x.rem_euclid(cs);
        }

        if tile.y >= cs {
            chunk.y += (tile.y / cs).floor() as i32;
            tile.y = tile.y.rem_euclid(cs);
        } else if tile.y < 0.0 {
            chunk.y -= ((-tile.y) / cs).ceil() as i32;
            tile.y = tile.y.rem_euclid(cs);
        }

        Self { chunk, tile }
    }

    /// Rounds tile coordinates and normalizes overflow.
    pub fn round(self) -> Self {
        Self::new(self.chunk, self.tile.round())
    }

    pub fn round_x(self) -> Self {
        Self::new(
            IVec2::new(self.chunk.x, self.chunk.y),
            Vec2::new(self.tile.x.round(), self.tile.y),
        )
    }

    pub fn round_y(self) -> Self {
        Self::new(
            IVec2::new(self.chunk.x, self.chunk.y),
            Vec2::new(self.tile.x, self.tile.y.round()),
        )
    }
}

impl Display for WorldTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match f.precision() {
            Some(p) => write!(f, "({:.p$}, {:.p$})", self.x(), self.y(), p = p),
            None => write!(f, "({}, {})", self.x(), self.y()),
        }
    }
}

impl From<WorldPosition> for WorldTransform {
    fn from(value: WorldPosition) -> Self {
        let (x, y) = value.tile.to_xy();
        Self::new(value.chunk, Vec2::new(x as f32, y as f32))
    }
}

impl From<Vec2> for WorldTransform {
    fn from(value: Vec2) -> Self {
        Self::from_xy(value.x, value.y)
    }
}

// Arithmetic
impl Add for WorldTransform {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.chunk + rhs.chunk, self.tile + rhs.tile)
    }
}

impl Sub for WorldTransform {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.chunk - rhs.chunk, self.tile - rhs.tile)
    }
}

impl AddAssign for WorldTransform {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for WorldTransform {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Add<Vec2> for WorldTransform {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        self + Self::from(rhs)
    }
}

impl Sub<Vec2> for WorldTransform {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        self - Self::from(rhs)
    }
}

impl AddAssign<Vec2> for WorldTransform {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs;
    }
}

impl SubAssign<Vec2> for WorldTransform {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs;
    }
}

// Rebase systems
pub fn apply_rebase(mut query: Query<(&mut Transform, &WorldTransform)>, base: Res<BaseChunk>) {
    if !base.is_changed() {
        return;
    }

    for (mut transform, world_transform) in query.iter_mut() {
        let pos = compute_transform(base.0, *world_transform);
        transform.translation = pos.extend(0.0);
    }
}

pub fn apply_world_transform(
    mut query: Query<(&mut Transform, &WorldTransform), Changed<WorldTransform>>,
    base: Res<BaseChunk>,
) {
    for (mut transform, world_transform) in query.iter_mut() {
        let pos = compute_transform(base.0, *world_transform);
        transform.translation = pos.extend(0.0);
    }
}

fn compute_transform(base: IVec2, position: WorldTransform) -> Vec2 {
    let chunk_size = CHUNK_SIZE as f32;
    let tile_size = TILE_SIZE as f32;

    let chunk_offset = position.chunk - base;
    let chunk_world = chunk_offset.as_vec2() * chunk_size * tile_size;
    chunk_world + position.tile * tile_size
}
