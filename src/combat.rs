use bevy::prelude::*;

use crate::physics::Rigidbody;

#[derive(Debug, Component)]
pub struct Gun {}

#[derive(Debug, Component)]
#[require(Rigidbody)]
pub struct Projectile {}

fn projectile_system() {}
