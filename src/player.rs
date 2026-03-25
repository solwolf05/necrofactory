use bevy::prelude::*;

use crate::physics::{Acceleration, GRAVITY, Grounded, Mass, Rigidbody, Velocity};

#[derive(Component)]
pub struct Player;

pub struct JetPackPlugin;

impl Plugin for JetPackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (jetpack_system, jetpack_refuel_system));
    }
}

#[derive(Debug, Default, Component)]
pub struct FuelTank {
    pub fuel: f32,
    pub max_fuel: f32,
    pub fuel_regen_rate: f32,
}

#[derive(Debug, Component)]
#[require(Rigidbody)]
#[require(FuelTank)]
#[require(JetpackControl)]
pub struct Jetpack {
    pub fuel_use_rate: f32,
    pub force: f32,
}

#[derive(Debug, Default, Component)]
pub struct JetpackControl {
    pub throttle: Vec2,
    pub hover: bool,
}

fn jetpack_system(
    query: Query<(
        &Jetpack,
        &mut FuelTank,
        &JetpackControl,
        &Velocity,
        &mut Acceleration,
        &Mass,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (jetpack, mut fuel_tank, control, velocity, mut acceleration, mass) in query {
        let mut throttle = control.throttle;
        if control.hover {
            throttle.y += (GRAVITY * mass.0) / jetpack.force;
            if control.throttle.length_squared() == 0.0 {
                throttle -= (velocity.0 * mass.0) / jetpack.force * 10.0;
            }
        }
        // Full throttle is 1 in each direction
        throttle = throttle.clamp_length_max(1.0);

        if throttle == Vec2::ZERO {
            continue;
        }

        let fuel_used = (jetpack.fuel_use_rate * throttle.length() * dt).min(fuel_tank.fuel);
        if fuel_used <= 0.0 {
            continue;
        }

        fuel_tank.fuel -= fuel_used;

        let thrust = throttle * jetpack.force;
        acceleration.0 += thrust / mass.0;
    }
}

fn jetpack_refuel_system(query: Query<&mut FuelTank, With<Grounded>>, time: Res<Time>) {
    let dt = time.delta_secs();

    for mut fuel_tank in query {
        fuel_tank.fuel = (fuel_tank.fuel + fuel_tank.fuel_regen_rate * dt).min(fuel_tank.max_fuel);
    }
}
