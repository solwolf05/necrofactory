use bevy::{math::I64Vec2, prelude::*};

use crate::{
    GameState,
    modding::Registry,
    physics::collision::{Aabb, CollisionEvent},
    world::{
        World, WorldTransform,
        tile::{Tile, TileDef},
    },
};

pub mod collision;

pub const GRAVITY: f32 = 9.8;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CollisionEvent>()
            .add_systems(FixedPreUpdate, reset_acceleration)
            .add_systems(
                FixedUpdate,
                (apply_gravity, apply_drag)
                    .before(PhysicsSet)
                    .run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                FixedUpdate,
                (
                    integrate_velocity,
                    solve_tile_collisions,
                    solve_entity_collisions,
                    log_collisions,
                )
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

fn reset_acceleration(mut query: Query<&mut Acceleration>) {
    for mut acc in &mut query {
        acc.0 = Vec2::ZERO;
    }
}

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

fn integrate_velocity(mut query: Query<(&mut Velocity, &Acceleration)>, time: Res<Time>) {
    let dt = time.delta_secs();

    for (mut vel, acc) in &mut query {
        vel.0 += acc.0 * dt;
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
    mut collisions: MessageWriter<CollisionEvent>,
) {
    let dt = time.delta_secs();
    let world = world.into_inner();
    let registry = registry.into_inner();

    for (entity, mut transform, mut vel, collider, restitution) in &mut query {
        let dt_vel = vel.0 * dt;

        let steps = Vec2::ONE;
        let step_vel = dt_vel / steps;

        let mut aabb = Aabb::new(transform.translation, collider.0);

        // y axis
        for _ in 0..steps.y as u32 {
            aabb.center.y = transform.translation.y + step_vel.y;

            let tiles = aabb.overlapping_tiles(world);
            if !tiles.is_empty() {
                let (tile_restitution, tile_friction) =
                    calculate_restitution_and_friction(&tiles, registry, restitution.0);

                vel.0.y *= -tile_restitution;
                vel.0.x *= 1.0 - tile_friction;

                if tiles.iter().any(|(_, pos)| {
                    pos.y < i64::from((transform.translation.y - (collider.0.y / 2.0)).ceil())
                }) {
                    commands.entity(entity).insert(Grounded);
                }

                collisions.write_batch(
                    tiles
                        .iter()
                        .map(|&(_, pos)| CollisionEvent::World { entity, tile: pos }),
                );

                break;
            } else {
                transform.translation.y += step_vel.y;
                commands.entity(entity).try_remove::<Grounded>();
            }
        }

        // x axis
        aabb.center.y = transform.translation.y;
        for _ in 0..steps.x as u32 {
            aabb.center.x = transform.translation.x + step_vel.x;

            let tiles = aabb.overlapping_tiles(world);
            if !tiles.is_empty() {
                let (tile_restitution, tile_friction) =
                    calculate_restitution_and_friction(&tiles, registry, restitution.0);

                vel.0.x *= -tile_restitution;
                vel.0.y *= 1.0 - tile_friction;

                collisions.write_batch(
                    tiles
                        .iter()
                        .map(|&(_, pos)| CollisionEvent::World { entity, tile: pos }),
                );

                break;
            } else {
                transform.translation.x += step_vel.x;
            }
        }
    }
}

fn calculate_restitution_and_friction(
    tiles: &[(&Tile, I64Vec2)],
    registry: &Registry<TileDef>,
    restitution: f32,
) -> (f32, f32) {
    let mut tile_restitution: f32 = 0.0;
    let mut tile_friction: f32 = 0.0;
    // Count is never 0
    let mut count = 0;

    for &(tile, _) in tiles {
        if let Some(tile_def) = registry.get(tile.id) {
            tile_restitution = tile_restitution.max(tile_def.restitution);
            tile_friction += tile_def.friction;
            count += 1;
        }
    }

    (tile_restitution * restitution, tile_friction / count as f32)
}

fn log_collisions(mut collisions: MessageReader<CollisionEvent>) {
    for collision in collisions.read() {
        info!("{:?}", collision);
    }
}

fn solve_entity_collisions(
    query: Query<(Entity, &WorldTransform, &Collider)>,
    mut collisions: MessageWriter<CollisionEvent>,
) {
    collisions.write_batch(
        query
            .iter()
            .map_windows(|[a, b]| {
                (
                    (a.0, Aabb::new(a.1.translation, a.2.0)),
                    (b.0, Aabb::new(b.1.translation, b.2.0)),
                )
            })
            .filter(|(a, b)| {
                a.1.center.chunk().distance_squared(b.1.center.chunk()) <= 1
                    && a.1.overlap_aabb(&b.1).is_some()
            })
            .map(|(a, b)| CollisionEvent::Rigidbody {
                entity: a.0,
                other: b.0,
            }),
    );
}
