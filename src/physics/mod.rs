use bevy::prelude::*;

use crate::{
    physics::collision::Aabb,
    world::{World, WorldTransform},
};

mod collision;

const GRAVITY: f32 = 9.8;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (integrate_velocity, apply_gravity, solve_tile_collisions).chain(),
        );
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
#[require(Mass, Restitution, Damping, Velocity, Acceleration)]
pub struct Rigidbody;

#[derive(Debug, Default, Component)]
pub struct Collider(pub Vec2);

fn apply_gravity(mut query: Query<&mut Acceleration>) {
    for mut acc in &mut query {
        acc.0.y -= GRAVITY;
    }
}

fn integrate_velocity(
    mut query: Query<(&mut Velocity, &mut Acceleration, &Damping)>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    for (mut vel, mut acc, damping) in &mut query {
        vel.0 += acc.0 * dt;

        vel.0 *= 1.0 - damping.0 * dt;

        acc.0 = Vec2::ZERO;
    }
}

fn solve_tile_collisions(
    mut query: Query<
        (&mut WorldTransform, &mut Velocity, &Collider, &Restitution),
        With<Rigidbody>,
    >,
    world: Res<World>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let world = world.into_inner();

    for (mut transform, mut vel, collider, restitution) in &mut query {
        let dt_vel = vel.0 * dt;
        let step_size = 1.0;

        let steps = (dt_vel.abs().max_element() / step_size).ceil() as u32;
        let step_vel = dt_vel / steps as f32;

        for _ in 0..steps {
            // x axis

            let mut new_pos = transform.translation;
            new_pos.x += step_vel.x;
            if Aabb::new(new_pos, collider.0).overlap_world(world) {
                vel.0.x *= -restitution.0;
                vel.0.y *= 0.9 * dt;
            } else {
                transform.translation.x = new_pos.x
            }

            // y axis

            let mut new_pos = transform.translation;
            new_pos.y += step_vel.y;
            if Aabb::new(new_pos, collider.0).overlap_world(world) {
                vel.0.y *= -restitution.0;
                vel.0.x *= 1.0 - 1.0 * dt;
            } else {
                transform.translation.y = new_pos.y
            }
        }
    }
}
