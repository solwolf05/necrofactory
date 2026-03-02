use bevy::prelude::*;

use crate::{
    math::I32F32,
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
