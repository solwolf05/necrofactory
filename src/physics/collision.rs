use std::fmt::Display;

use bevy::{math::I64Vec2, prelude::*};

use crate::{
    math::{Hybrid, HybridVec2},
    world::World,
};

#[derive(Debug)]
pub struct Aabb {
    pub left: Hybrid,
    pub right: Hybrid,
    pub up: Hybrid,
    pub down: Hybrid,
}

impl Aabb {
    pub fn new(pos: HybridVec2, size: Vec2) -> Self {
        Self {
            left: pos.x - size.x / 2.0,
            right: pos.x + size.x / 2.0 - 0.0001,
            up: pos.y + size.y / 2.0 - 0.0001,
            down: pos.y - size.y / 2.0,
        }
    }

    pub fn overlap_aabb(&self, other: &Aabb) -> bool {
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

impl Display for Aabb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "min: ({}, {}), max: ({}, {})",
            self.left, self.down, self.right, self.up
        )
    }
}
