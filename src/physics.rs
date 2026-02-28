use bevy::prelude::*;

use crate::{
    math::I32F32,
    modding::Id,
    world::{World, WorldTransform},
};

const GRAVITY: f32 = -9.8;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, (run_physics, collision_system).chain());
    }
}

#[derive(Debug, Default, Component)]
pub struct Rigidbody {
    pub mass: f32,
    pub acceleration: Vec2,
    pub velocity: Vec2,
    pub restitution: f32,
}

impl Rigidbody {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            acceleration: Vec2::ZERO,
            velocity: Vec2::ZERO,
            restitution: 0.0,
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

fn collision_system(
    mut query: Query<(&mut Rigidbody, &mut WorldTransform)>,
    world: Res<World>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    for (mut body, mut transform) in query.iter_mut() {
        //
        // ----- X AXIS -----
        //
        let dx = body.velocity.x * dt;
        transform.translation.x += dx;

        let min_x = (transform.translation.x + I32F32::MIN_POSITIVE)
            .floor()
            .into();
        let max_x = (transform.translation.x + I32F32::ONE - I32F32::MIN_POSITIVE)
            .floor()
            .into();
        let min_y = (transform.translation.y + I32F32::MIN_POSITIVE)
            .floor()
            .into();
        let max_y = (transform.translation.y + I32F32::ONE - I32F32::MIN_POSITIVE)
            .floor()
            .into();

        if body.velocity.x > 0.0 {
            for y in min_y..=max_y {
                let tile = IVec2::new(max_x, y);
                if let Some(t) = world.get_tile(tile.into())
                    && t.id != Id::ZERO
                {
                    transform.translation.x = (tile.x - 1).into();
                    body.velocity.x = body.velocity.x * -body.restitution;
                    break;
                }
            }
        } else if body.velocity.x < 0.0 {
            for y in min_y..=max_y {
                let tile = IVec2::new(min_x, y);
                if let Some(t) = world.get_tile(tile.into())
                    && t.id != Id::ZERO
                {
                    transform.translation.x = (tile.x + 1).into();
                    body.velocity.x = body.velocity.x * -body.restitution;
                    break;
                }
            }
        }

        //
        // ----- Y AXIS -----
        //
        let dy = body.velocity.y * dt;
        transform.translation.y += dy;

        let min_x = (transform.translation.x + I32F32::MIN_POSITIVE)
            .floor()
            .into();
        let max_x = (transform.translation.x + I32F32::ONE - I32F32::MIN_POSITIVE)
            .floor()
            .into();
        let min_y = (transform.translation.y + I32F32::MIN_POSITIVE)
            .floor()
            .into();
        let max_y = (transform.translation.y + I32F32::ONE - I32F32::MIN_POSITIVE)
            .floor()
            .into();

        if body.velocity.y > 0.0 {
            for x in min_x..=max_x {
                let tile = IVec2::new(x, max_y);
                if let Some(t) = world.get_tile(tile.into())
                    && t.id != Id::ZERO
                {
                    transform.translation.y = (tile.y - 1).into();
                    body.velocity.y = body.velocity.y * -body.restitution;
                    break;
                }
            }
        } else if body.velocity.y < 0.0 {
            for x in min_x..=max_x {
                let tile = IVec2::new(x, min_y);
                if let Some(t) = world.get_tile(tile.into())
                    && t.id != Id::ZERO
                {
                    transform.translation.y = (tile.y + 1).into();
                    body.velocity.y = body.velocity.y * -body.restitution;
                    break;
                }
            }
        }
    }
}
