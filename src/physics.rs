use bevy::prelude::*;

use crate::{
    modding::Id,
    world::{TILE_SIZE, World, WorldTransform},
};

const GRAVITY: f32 = -9.8;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (run_physics, collision_system, update_transforms).chain(),
        );
    }
}

#[derive(Debug, Default, Component)]
pub struct Rigidbody {
    pub mass: f32,
    pub acceleration: Vec2,
    pub velocity: Vec2,
}

impl Rigidbody {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            acceleration: Vec2::ZERO,
            velocity: Vec2::ZERO,
        }
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.acceleration += force / self.mass;
    }
}

fn run_physics(mut query: Query<&mut Rigidbody>, time: Res<Time<Fixed>>) {
    let dt = time.delta_secs();
    for mut body in query.iter_mut() {
        body.acceleration += Vec2::Y * GRAVITY;

        let acceleration = body.acceleration;
        body.velocity += acceleration * dt;

        body.acceleration = Vec2::ZERO;
    }
}

fn update_transforms(mut query: Query<(&mut WorldTransform, &Rigidbody)>, time: Res<Time<Fixed>>) {
    let dt = time.delta_secs();
    for (mut transform, body) in query.iter_mut() {
        transform.translation += body.velocity * dt;
    }
}

fn collision_system(mut query: Query<(&mut Rigidbody, &mut WorldTransform)>, world: Res<World>) {
    for (mut body, mut transform) in query.iter_mut() {}
}
