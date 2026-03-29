use bevy::prelude::*;

use crate::{
    GameState,
    modding::Registry,
    physics::collision::Aabb,
    world::{World, WorldTransform, tile::TileDef},
};

mod collision;

pub const GRAVITY: f32 = 9.8;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (apply_gravity, apply_drag)
                .before(PhysicsSet)
                .run_if(in_state(GameState::InGame)),
        )
        .add_systems(
            FixedUpdate,
            (integrate_velocity, solve_tile_collisions)
                .chain()
                .in_set(PhysicsSet)
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsSet;

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
pub struct Drag(pub f32);

#[derive(Debug, Default, Component)]
#[require(WorldTransform, Drag)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Default, Component)]
#[require(Velocity)]
pub struct Acceleration(pub Vec2);

#[derive(Debug, Default, Component)]
#[require(Mass, Restitution, Drag, Velocity, Acceleration)]
pub struct Rigidbody;

#[derive(Debug, Default, Component)]
pub struct Collider(pub Vec2);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

fn apply_gravity(mut query: Query<&mut Acceleration>) {
    for mut acc in &mut query {
        acc.0.y -= GRAVITY;
    }
}

fn apply_drag(query: Query<(&mut Acceleration, &Velocity, &Drag, &Mass)>) {
    for (mut acc, vel, drag, mass) in query {
        let force = drag.0 * vel.0 * vel.0.length() / mass.0;
        acc.0 -= force;
    }
}

fn integrate_velocity(mut query: Query<(&mut Velocity, &mut Acceleration)>, time: Res<Time>) {
    let dt = time.delta_secs();

    for (mut vel, mut acc) in &mut query {
        vel.0 += acc.0 * dt;

        acc.0 = Vec2::ZERO;
    }
}

fn solve_tile_collisions(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut WorldTransform,
            &mut Velocity,
            &Collider,
            &Restitution,
        ),
        With<Rigidbody>,
    >,
    world: Res<World>,
    registry: Res<Registry<TileDef>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let world = world.into_inner();

    for (entity, mut transform, mut vel, collider, restitution) in &mut query {
        let dt_vel = vel.0 * dt;

        let steps = dt_vel.abs().ceil();
        let step_vel = dt_vel / steps;

        for _ in 0..steps.x as u32 {
            // x axis

            let mut new_pos = transform.translation;
            new_pos.x += step_vel.x;
            let tiles = Aabb::new(new_pos, collider.0).overlapping_tiles(world);
            if tiles.is_some() {
                let friction: f32 = tiles
                    .iter()
                    .flat_map(|t| registry.get(t.id))
                    .map(|t| t.friction)
                    .sum::<f32>()
                    * 0.5;

                vel.0.x *= -restitution.0;
                vel.0.y *= 1.0 - friction;
                break;
            } else {
                transform.translation.x = new_pos.x;
            }
        }

        for _ in 0..steps.y as u32 {
            // y axis

            let mut new_pos = transform.translation;
            new_pos.y += step_vel.y;
            let tiles = Aabb::new(new_pos, collider.0).overlapping_tiles(world);
            if tiles.is_some() {
                let friction: f32 = tiles
                    .iter()
                    .flat_map(|t| registry.get(t.id))
                    .map(|t| t.friction)
                    .sum::<f32>()
                    * 0.5;

                vel.0.y *= -restitution.0;
                vel.0.x *= 1.0 - friction;
                if tiles.is_bottom_some() {
                    commands.entity(entity).insert(Grounded);
                }
                break;
            } else {
                transform.translation.y = new_pos.y;
                commands.entity(entity).try_remove::<Grounded>();
            }
        }
    }
}
