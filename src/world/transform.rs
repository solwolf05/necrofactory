use bevy::prelude::*;
use fixed::traits::ToFixed;

use crate::{
    math::FixedVec2,
    world::{CHUNK_SIZE, TILE_SIZE},
};

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
    pub translation: FixedVec2,
}

impl WorldTransform {
    pub fn from_translation(translation: FixedVec2) -> Self {
        Self { translation }
    }

    pub fn from_chunk(chunk: IVec2) -> Self {
        Self::from_translation(FixedVec2::new(
            (chunk.x * CHUNK_SIZE as i32).into(),
            (chunk.y * CHUNK_SIZE as i32).into(),
        ))
    }

    pub fn from_tile(tile: Vec2) -> Self {
        Self::from_translation(FixedVec2::new(tile.x.to_fixed(), tile.y.to_fixed()))
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
    let offset = world_transform.translation - base.0 * CHUNK_SIZE as i32;
    let pos = offset * TILE_SIZE as i64;
    transform.translation = Vec2::from(pos).extend(0.0);
}
