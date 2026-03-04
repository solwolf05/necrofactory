use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

use bevy::prelude::*;

use crate::world::{CHUNK_SIZE, TILE_SIZE};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RebaseSet;

/// The chunk position that the world is centered around
#[derive(Debug, Default, Resource)]
pub struct BaseChunk(pub IVec2);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct HybridVec2 {
    pub chunk: IVec2,
    pub tile: Vec2,
}

impl HybridVec2 {
    pub fn new(chunk: IVec2, tile: Vec2) -> Self {
        Self { chunk, tile }.normalize()
    }

    pub fn from_chunk(chunk: IVec2) -> Self {
        Self {
            chunk,
            tile: Vec2::default(),
        }
    }

    pub fn from_tile(tile: Vec2) -> Self {
        Self {
            chunk: IVec2::default(),
            tile,
        }
        .normalize()
    }

    pub fn normalize(self) -> Self {
        let chunk_offset = self
            .tile
            .div_euclid(Vec2::splat(TILE_SIZE as f32))
            .as_ivec2();
        let tile = self.tile.rem_euclid(Vec2::splat(TILE_SIZE as f32));
        Self {
            chunk: self.chunk + chunk_offset,
            tile,
        }
    }

    pub fn round(self) -> Self {
        Self::new(self.chunk, self.tile.round())
    }

    pub fn floor(self) -> Self {
        Self::new(self.chunk, self.tile.floor())
    }

    pub fn ceil(self) -> Self {
        Self::new(self.chunk, self.tile.ceil())
    }
}

impl Display for HybridVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Vec2::from(*self).fmt(f)
    }
}

impl From<Vec2> for HybridVec2 {
    fn from(value: Vec2) -> Self {
        Self::from_tile(value)
    }
}

impl From<HybridVec2> for Vec2 {
    fn from(value: HybridVec2) -> Self {
        (value.chunk * CHUNK_SIZE as i32).as_vec2() + value.tile
    }
}

impl From<IVec2> for HybridVec2 {
    fn from(value: IVec2) -> Self {
        let chunk = value.div_euclid(IVec2::splat(CHUNK_SIZE as i32));
        let tile = value.rem_euclid(IVec2::splat(CHUNK_SIZE as i32)).as_vec2();
        Self::new(chunk, tile)
    }
}

impl From<HybridVec2> for IVec2 {
    fn from(value: HybridVec2) -> Self {
        (value.chunk * CHUNK_SIZE as i32) + value.tile.as_ivec2()
    }
}

impl Add for HybridVec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.chunk + rhs.chunk, self.tile + rhs.tile)
    }
}

impl Add<Vec2> for HybridVec2 {
    type Output = Self;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::new(self.chunk, self.tile + rhs)
    }
}

impl Sub for HybridVec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.chunk - rhs.chunk, self.tile - rhs.tile)
    }
}

impl Sub<Vec2> for HybridVec2 {
    type Output = Self;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::new(self.chunk, self.tile + rhs)
    }
}

impl Mul<i32> for HybridVec2 {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(self.chunk * rhs, self.tile * rhs as f32)
    }
}

impl Div<i32> for HybridVec2 {
    type Output = Self;

    fn div(self, rhs: i32) -> Self::Output {
        Self::new(self.chunk / rhs, self.tile / rhs as f32)
    }
}

impl AddAssign for HybridVec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.chunk += rhs.chunk;
        self.tile += rhs.tile;
        *self = self.normalize();
    }
}

impl AddAssign<Vec2> for HybridVec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.tile += rhs;
        *self = self.normalize();
    }
}

impl SubAssign for HybridVec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.chunk -= rhs.chunk;
        self.tile -= rhs.tile;
        *self = self.normalize();
    }
}

impl SubAssign<Vec2> for HybridVec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        self.tile -= rhs;
        *self = self.normalize();
    }
}

/// Used for rendering and other operations that require floating-point coordinates.
/// Separates the world into chunks and tiles to avoid floating point precision issues far from origin.
#[derive(Debug, Default, Clone, Copy, Component)]
#[require(Transform)]
pub struct WorldTransform {
    pub translation: HybridVec2,
}

impl WorldTransform {
    pub fn from_translation(translation: HybridVec2) -> Self {
        Self { translation }
    }

    pub fn from_chunk(chunk: IVec2) -> Self {
        Self::from_translation(HybridVec2::from_chunk(chunk))
    }

    pub fn from_tile(tile: Vec2) -> Self {
        Self::from_translation(HybridVec2::from_tile(tile))
    }
}

// Rebase systems
pub fn apply_rebase(mut query: Query<(&mut Transform, &WorldTransform)>, base: Res<BaseChunk>) {
    if !base.is_changed() {
        return;
    }

    for (mut transform, world_transform) in query.iter_mut() {
        rebase(&base, &mut transform, world_transform);
    }
}

pub fn apply_world_transform(
    mut query: Query<(&mut Transform, &WorldTransform), Changed<WorldTransform>>,
    base: Res<BaseChunk>,
) {
    for (mut transform, world_transform) in query.iter_mut() {
        rebase(&base, &mut transform, world_transform);
    }
}

fn rebase(base: &BaseChunk, transform: &mut Transform, world_transform: &WorldTransform) {
    let world_pos =
        world_transform.translation - HybridVec2::from_chunk(base.0 * CHUNK_SIZE as i32);
    let pos = world_pos * TILE_SIZE as i32;
    transform.translation = Vec2::from(pos).extend(0.0);
}
