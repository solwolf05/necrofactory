use bevy::math::{I64Vec2, Vec2};

use crate::{math::HybridVec2, world::World};

#[derive(Debug)]
pub struct Aabb {
    center: HybridVec2,
    half_extents: Vec2,
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
        let min = self.center - self.half_extents;
        let max = self.center + self.half_extents - 0.0001;

        for x in min.x.round().into()..=max.x.round().into() {
            for y in min.y.round().into()..=max.y.round().into() {
                let pos = I64Vec2::new(x, y);
                if world.contains_tile(pos) {
                    return true;
                }
            }
        }

        false
    }

    pub fn sweep_point(
        &self,
        origin: HybridVec2,
        delta: Vec2,
        padding: Vec2,
    ) -> Option<SweepContact> {
        let inv_delta = Vec2::new(
            if delta.x != 0.0 {
                1.0 / delta.x
            } else {
                f32::INFINITY
            },
            if delta.y != 0.0 {
                1.0 / delta.y
            } else {
                f32::INFINITY
            },
        );

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
        if delta.length_squared() == 0.0 {
            return self.overlap_aabb(other).map(|c| SweepContact {
                point: c.point,
                normal: c.mtv.normalize_or_zero(),
                time: 0.0,
            });
        }

        self.sweep_point(other.center, delta, other.half_extents)
    }
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

pub fn rebound(delta: Vec2, time: f32, normal: Vec2, restitution: f32) -> Vec2 {
    let first = delta * time;
    let last = delta * (1.0 - time);
    let last_reflected = last - last.dot(normal) * normal * 2.0 * restitution;

    first + last_reflected
}
