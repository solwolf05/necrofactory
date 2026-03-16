use std::fmt::Display;

use bevy::{math::I64Vec2, prelude::*};

use crate::{
    math::{Hybrid, HybridVec2},
    world::World,
};

#[derive(Debug)]
pub struct Aabb {
    center: HybridVec2,
    half_extents: Vec2,
}

impl Aabb {
    pub fn new(center: HybridVec2, size: Vec2) -> Self {
        Self {
            center,
            half_extents: size / 2.0,
        }
    }

    pub fn overlap_aabb(&self, other: &Aabb) -> bool {
        let self_max = self.center + self.half_extents;
        let self_min = self.center - self.half_extents;

        let other_max = self.center + self.half_extents;
        let other_min = self.center - self.half_extents;

        let x_overlap = self_min.x < other_max.x && self_max.x > other_min.x;
        let y_overlap = self_min.y < other_max.y && self_max.y > other_min.y;

        x_overlap && y_overlap
    }
}

#[derive(Debug)]
pub struct Rect {
    pub left: Hybrid,
    pub right: Hybrid,
    pub up: Hybrid,
    pub down: Hybrid,
}

impl Rect {
    pub fn new(pos: HybridVec2, size: Vec2) -> Self {
        Self {
            left: pos.x - size.x / 2.0,
            right: pos.x + size.x / 2.0 - 0.0001,
            up: pos.y + size.y / 2.0 - 0.0001,
            down: pos.y - size.y / 2.0,
        }
    }

    pub fn overlap_aabb(&self, other: &Rect) -> bool {
        let x_overlap = self.left < other.right && self.right > other.left;
        let y_overlap = self.down < other.up && self.up > other.down;

        x_overlap && y_overlap
    }

    pub fn overlap_world(&self, world: &World) -> bool {
        for x in self.left.round().into()..=self.right.round().into() {
            for y in self.down.round().into()..=self.up.round().into() {
                let pos = I64Vec2::new(x, y);
                if world.contains_tile(pos) {
                    return true;
                }
            }
        }

        false
    }
}

impl Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "min: ({}, {}), max: ({}, {})",
            self.left, self.down, self.right, self.up
        )
    }
}
