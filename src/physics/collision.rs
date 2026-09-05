use bevy::{math::I64Vec2, prelude::*};
use smallvec::SmallVec;

use crate::{
    math::{Hybrid, HybridVec2},
    modding::Registry,
    world::{
        World,
        tile::{Tile, TileDef},
    },
};

#[derive(Debug, Message)]
pub enum CollisionEvent {
    Rigidbody { entity: Entity, other: Entity },
    World { entity: Entity, tile: I64Vec2 },
}

impl CollisionEvent {
    pub fn entity(&self) -> Entity {
        match self {
            CollisionEvent::Rigidbody { entity, .. } => *entity,
            CollisionEvent::World { entity, .. } => *entity,
        }
    }
}

#[derive(Debug)]
pub struct Aabb {
    pub center: HybridVec2,
    pub half_extents: Vec2,
}

impl Aabb {
    pub fn new(pos: HybridVec2, size: Vec2) -> Self {
        Self {
            center: pos,
            half_extents: size / 2.0,
        }
    }

    pub fn from_tile(pos: I64Vec2) -> Self {
        Self {
            center: pos.into(),
            half_extents: Vec2::splat(0.5),
        }
    }

    pub fn overlap_point(&self, point: HybridVec2) -> Option<Contact> {
        let d = point - self.center;
        let penetration = self.half_extents - Vec2::from(d.abs());

        if penetration.x < 0.0 || penetration.y < 0.0 {
            return None;
        }

        if penetration.x < penetration.y {
            let normal = Vec2::X * d.x.signum();
            Some(Contact {
                point: point.with_x(self.center.x + self.half_extents.x * normal.x),
                normal,
                mtv: normal * penetration.x,
            })
        } else {
            let normal = Vec2::Y * d.y.signum();
            Some(Contact {
                point: point.with_y(self.center.y + self.half_extents.y * normal.y),
                normal,
                mtv: normal * penetration.y,
            })
        }
    }

    pub fn overlap_aabb(&self, other: &Aabb) -> Option<Contact> {
        let d = other.center - self.center;
        let penetration = (self.half_extents + other.half_extents) - Vec2::from(d.abs());

        if penetration.x < 0.0 || penetration.y < 0.0 {
            return None;
        }

        if penetration.x < penetration.y {
            let normal = Vec2::X * d.x.signum();
            Some(Contact {
                point: other
                    .center
                    .with_x(self.center.x + self.half_extents.x * normal.x),
                normal,
                mtv: normal * penetration.x,
            })
        } else {
            let normal = Vec2::Y * d.y.signum();
            Some(Contact {
                point: other
                    .center
                    .with_y(self.center.y + self.half_extents.y * normal.y),
                normal,
                mtv: normal * penetration.y,
            })
        }
    }

    pub fn overlap_world(&self, world: &World) -> bool {
        let min = self.center - self.half_extents + 0.5;
        let max = self.center + self.half_extents + 0.5 - 0.0001;

        for x in min.x.floor().into()..=max.x.floor().into() {
            for y in min.y.floor().into()..=max.y.floor().into() {
                let pos = I64Vec2::new(x, y);
                if world.contains_tile(pos) {
                    return true;
                }
            }
        }

        false
    }

    pub fn overlapping_tiles<'w>(&self, world: &'w World) -> SmallVec<[(&'w Tile, I64Vec2); 4]> {
        let min = I64Vec2::from((self.center - self.half_extents).round());
        let max = I64Vec2::from((self.center + self.half_extents - 0.0001).round());

        (min.x..=max.x)
            .flat_map(move |x| {
                (min.y..=max.y).flat_map(move |y| {
                    let pos = I64Vec2::new(x, y);
                    world.get_tile(pos).map(|tile| (tile, pos))
                })
            })
            .collect::<SmallVec<_>>()
    }

    pub fn sweep_point(
        &self,
        origin: HybridVec2,
        delta: Vec2,
        padding: Vec2,
    ) -> Option<SweepContact> {
        let inv_delta = 1.0 / delta;

        let min = self.center - (self.half_extents + padding);
        let max = self.center + (self.half_extents + padding);

        let t1 = (min - origin).to_vec2() * inv_delta;
        let t2 = (max - origin).to_vec2() * inv_delta;

        let t_near = t1.min(t2);
        let t_far = t1.max(t2);

        if t_near.x > t_far.y || t_near.y > t_far.x {
            return None;
        }

        let near = t_near.max_element();
        let far = t_far.min_element();

        if near >= 1.0 || near < 0.0 || far <= 0.0 {
            return None;
        }

        let time = near.clamp(0.0, 1.0);

        Some(SweepContact {
            point: origin + delta * time,
            normal: (t_near.x > t_near.y)
                .then(|| Vec2::X * -delta.x.signum())
                .unwrap_or(Vec2::Y * -delta.y.signum()),
            time,
        })
    }

    pub fn sweep_aabb(&self, other: &Aabb, delta: Vec2) -> Option<SweepContact> {
        self.sweep_point(other.center, delta, other.half_extents)
    }
}

pub fn get_tile(world: &World, x: Hybrid, y: Hybrid) -> Option<&Tile> {
    let pos = I64Vec2::new(x.round().into(), y.round().into());
    world.get_tile(pos)
}

#[derive(Debug)]
pub struct Contact {
    pub point: HybridVec2,
    pub normal: Vec2,
    pub mtv: Vec2,
}

#[derive(Debug)]
pub struct SweepContact {
    pub point: HybridVec2,
    pub normal: Vec2,
    pub time: f32,
}

#[derive(Debug, Default)]
pub struct OverlappingTiles<'w>(Vec<(&'w Tile, I64Vec2)>);

impl<'w> OverlappingTiles<'w> {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'w Tile, I64Vec2)> {
        self.0.iter().map(|(tile, pos)| (*tile, *pos))
    }

    pub fn iter_defs<'a>(
        &self,
        registry: &'a Registry<TileDef>,
    ) -> impl Iterator<Item = &'a TileDef> {
        self.iter().flat_map(|(t, _)| registry.get(t.handle))
    }
}
