use bevy::{math::I64Vec2, prelude::*};

use crate::{
    math::{FixedVec2, I32F32},
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

fn collision_system(
    mut query: Query<(&mut Rigidbody, &mut WorldTransform)>,
    world: Res<World>,
    time: Res<Time<Fixed>>,
) {
    let delta = time.delta_secs();
    for (mut body, mut transform) in query.iter_mut() {
        let next_pos = transform.translation + body.velocity * delta;

        let start = IVec2::from(transform.translation) - 1;
        let end = IVec2::from(transform.translation /* - I32F32::from_bits(1) */) + 1;

        let mut horizontal: Vec<IVec2> = Vec::new();
        let mut vertical: Vec<IVec2> = Vec::new();
        let mut diagonal: Option<IVec2> = Option::None;

        if body.velocity.x > 0.0 {
            for y in (start.y + 1)..start.y {
                let pos = end.with_y(y);
                if let Some(tile) = world.get_tile(pos.into())
                    && tile.id != Id::ZERO
                {
                    vertical.push(pos);
                }
            }
        } else if body.velocity.x < 0.0 {
            for y in (start.y + 1)..start.y {
                let pos = start.with_y(y);
                if let Some(tile) = world.get_tile(pos.into())
                    && tile.id != Id::ZERO
                {
                    vertical.push(pos);
                }
            }
        }

        if body.velocity.y > 0.0 {
            for x in (start.x + 1)..start.x {
                let pos = end.with_x(x);
                if let Some(tile) = world.get_tile(pos.into())
                    && tile.id != Id::ZERO
                {
                    horizontal.push(pos);
                }
            }
        } else if body.velocity.y < 0.0 {
            for x in (start.x + 1)..start.x {
                let pos = start.with_x(x);
                if let Some(tile) = world.get_tile(pos.into())
                    && tile.id != Id::ZERO
                {
                    horizontal.push(pos);
                }
            }
        }

        if body.velocity.x < 0.0 && body.velocity.y > 0.0 {
            let pos = start;
            if let Some(tile) = world.get_tile(pos.into())
                && tile.id != Id::ZERO
            {
                diagonal = Some(pos);
            }
        } else if body.velocity.x > 0.0 && body.velocity.y > 0.0 {
            let pos = start.with_x(end.x);
            if let Some(tile) = world.get_tile(pos.into())
                && tile.id != Id::ZERO
            {
                diagonal = Some(pos);
            }
        } else if body.velocity.x < 0.0 && body.velocity.y < 0.0 {
            let pos = start.with_y(end.y);
            if let Some(tile) = world.get_tile(pos.into())
                && tile.id != Id::ZERO
            {
                diagonal = Some(pos);
            }
        } else if body.velocity.x > 0.0 && body.velocity.y < 0.0 {
            let pos = end;
            if let Some(tile) = world.get_tile(pos.into())
                && tile.id != Id::ZERO
            {
                diagonal = Some(pos);
            }
        }

        for pos in vertical {
            if next_pos.x + 1 > pos.x && next_pos.x < pos.x + 1 {
                if body.velocity.x >= 0.0 {
                    body.velocity.x = (I32F32::from(pos.x) - (transform.translation.x + 1)).into();
                } else if body.velocity.x < 0.0 {
                    body.velocity.x =
                        (I32F32::from(pos.x + 1) - (transform.translation.x + 1)).into();
                }
            }
        }
    }
}
