use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use bevy::prelude::*;

use crate::world::{CHUNK_SIZE, TILE_SIZE, WorldPosition};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RebaseSet;

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

    let chunk = position.chunk - base;
    let bevy_chunk = chunk.as_vec2() * chunk_size * tile_size;
    let position = bevy_chunk + position.tile * tile_size;
    position
}

/// The chunk position that the world is centered around
#[derive(Debug, Default, Resource)]
pub struct BaseChunk(pub IVec2);

/// Used for rendering and other operations that require floating-point coordinates.
#[derive(Debug, Default, Clone, Copy, Component)]
#[require(Transform)]
pub struct WorldTransform {
    pub chunk: IVec2,
    pub tile: Vec2,
}

impl WorldTransform {
    pub fn new(chunk: IVec2, tile: Vec2) -> Self {
        Self { chunk, tile }
    }

    pub fn from_chunk(chunk: IVec2) -> Self {
        Self::new(chunk, Vec2::ZERO)
    }

    pub fn from_tile(tile: Vec2) -> Self {
        Self::new(IVec2::ZERO, tile)
    }

    /// Panics if out of bounds, which hopefully isnt a problem unless the player goes really far away
    pub fn from_xy(x: f32, y: f32) -> Self {
        let cs = CHUNK_SIZE as f32;

        let chunk_x = x.div_euclid(cs) as i32;
        let chunk_y = y.div_euclid(cs) as i32;

        let tile_x = x.rem_euclid(cs);
        let tile_y = y.rem_euclid(cs);

        Self::new(
            IVec2::new(chunk_x.try_into().unwrap(), chunk_y.try_into().unwrap()),
            Vec2::new(tile_x, tile_y),
        )
    }

    pub fn x(&self) -> f32 {
        (self.chunk.x * CHUNK_SIZE as i32) as f32 + self.tile.x
    }

    pub fn y(&self) -> f32 {
        (self.chunk.y * CHUNK_SIZE as i32) as f32 + self.tile.y
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x(), self.y())
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
        Self {
            chunk: value.chunk,
            tile: Vec2::new(x as f32, y as f32),
        }
    }
}

impl From<Vec2> for WorldTransform {
    fn from(value: Vec2) -> Self {
        Self::from_xy(value.x, value.y)
    }
}

impl Add for WorldTransform {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let chunk_size = CHUNK_SIZE as f32;
        let cx = self.chunk.x + rhs.chunk.x;
        let cy = self.chunk.y + rhs.chunk.y;

        let tx = self.tile.x + rhs.tile.x;
        let ty = self.tile.y + rhs.tile.y;

        // Handle overflow for x
        let (tx, cx) = if tx >= chunk_size {
            (tx - chunk_size, cx + 1)
        } else if tx < 0.0 {
            (tx + chunk_size, cx - 1)
        } else {
            (tx, cx)
        };

        // Handle overflow for y
        let (ty, cy) = if ty >= chunk_size {
            (ty - chunk_size, cy + 1)
        } else if ty < 0.0 {
            (ty + chunk_size, cy - 1)
        } else {
            (ty, cy)
        };

        WorldTransform::new(IVec2::new(cx, cy), Vec2::new(tx, ty))
    }
}

impl Sub for WorldTransform {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let chunk_size = CHUNK_SIZE as f32;
        let cx = self.chunk.x as f32 - rhs.chunk.x as f32;
        let cy = self.chunk.y as f32 - rhs.chunk.y as f32;

        let tx = self.tile.x - rhs.tile.x;
        let ty = self.tile.y - rhs.tile.y;

        // Handle underflow for x
        let (tx, cx) = if tx >= chunk_size {
            (tx - chunk_size, cx + 1.0)
        } else if tx < 0.0 {
            (tx + chunk_size, cx - 1.0)
        } else {
            (tx, cx)
        };

        // Handle underflow for y
        let (ty, cy) = if ty >= chunk_size {
            (ty - chunk_size, cy + 1.0)
        } else if ty < 0.0 {
            (ty + chunk_size, cy - 1.0)
        } else {
            (ty, cy)
        };

        WorldTransform::new(IVec2::new(cx as i32, cy as i32), Vec2::new(tx, ty))
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
