use bevy::prelude::*;

use crate::{
    modding::Id,
    world::{World, WorldTransform},
};

const GRAVITY: f32 = 9.8;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, (run_physics, collision_system).chain());
    }
}

#[derive(Debug, Component)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Debug, Default, Component)]
pub struct Restitution(pub f32);

#[derive(Debug, Default, Component)]
pub struct Damping(pub f32);

#[derive(Debug, Default, Component)]
#[require(WorldTransform, Damping)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Default, Component)]
#[require(Velocity)]
pub struct Acceleration(pub Vec2);

#[derive(Debug, Default, Component)]
#[require(Mass, Restitution, Velocity, Acceleration)]
pub struct Rigidbody;

#[derive(Debug, Default, Component)]
pub struct Aabb(pub Vec2);

impl Aabb {
    pub fn half_x(&self) -> f32 {
        self.0.x / 2.0
    }

    pub fn half_y(&self) -> f32 {
        self.0.y / 2.0
    }
}

fn run_physics(
    mut query: Query<(
        &mut WorldTransform,
        &mut Velocity,
        &mut Acceleration,
        &Damping,
    )>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity, mut acceleration, damping) in query.iter_mut() {
        velocity.0 *= 1.0 - damping.0 * dt;
        acceleration.0 += Vec2::NEG_Y * GRAVITY;
        velocity.0 += acceleration.0 * dt;
        transform.translation += velocity.0 * dt;

        acceleration.0 = Vec2::ZERO;
    }
}

fn collision_system(
    mut query: Query<(&mut Rigidbody, &mut WorldTransform)>,
    world: Res<World>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();
}

struct ComputedAabb {
    pos: Vec2,
    half: Vec2,
}

impl ComputedAabb {
    pub fn intersect_point(&self, pos: Vec2) -> Option<Intersection> {
        let dx = pos.x - self.pos.x;
        let px = self.half.x - dx.abs();
        if px <= 0.0 {
            return None;
        }

        let dy = pos.y - self.pos.y;
        let py = self.half.y - dy.abs();
        if py <= 0.0 {
            return None;
        }

        // let d = pos - self.pos;
        // let p = self.half - d.abs();
        // if p.x > 0.0 || p.y > 0.0 {
        //     return None;
        // }

        if px < py {
            Some(Intersection {
                pos: pos.with_x(self.pos.x + (self.half.x * dx.signum())),
                delta: Vec2::X * px * dx.signum(),
            })
        } else {
            Some(Intersection {
                pos: pos.with_y(self.pos.y + (self.half.y * dy.signum())),
                delta: Vec2::Y * py * dy.signum(),
            })
        }
    }

    pub fn intersect_segment(&self, pos: Vec2, delta: Vec2) -> Option<ContinuousIntersection> {
        let scale = 1.0 / delta;
        let near = (self.pos - scale.signum() * self.half - pos) * scale;
        let far = (self.pos + scale.signum() * self.half - pos) * scale;

        if near.x > far.y || near.y > far.x {
            return None;
        }

        let near = near.max_element();
        let far = far.max_element();

        if near >= 1.0 || far <= 0.0 {
            return None;
        }

        let time = near.clamp(0.0, 1.0);
        let delta = (1.0 - time) * -delta;
        let pos = pos + delta * time;
        Some(ContinuousIntersection { pos, delta, time })
    }

    pub fn intersect_aabb(&self, other: ComputedAabb) -> Option<Intersection> {
        todo!()
    }
}

struct Intersection {
    pub pos: Vec2,
    pub delta: Vec2,
}

struct ContinuousIntersection {
    pub pos: Vec2,
    pub delta: Vec2,
    pub time: f32,
}
