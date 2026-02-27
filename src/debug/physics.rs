use crate::{debug::DebugText, physics::Rigidbody, player::Player};
use bevy::prelude::*;

use crate::AppState;

pub struct PhysicsDebugPlugin;

impl Plugin for PhysicsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup)
            .add_systems(Update, update_text.run_if(in_state(AppState::InGame)));
    }
}

#[derive(Component)]
pub struct AccelerationText;

#[derive(Component)]
pub struct VelocityText;

fn setup(mut commands: Commands, text_root: Query<Entity, With<DebugText>>) {
    let text_root = text_root.single().unwrap();
    commands.entity(text_root).with_children(|parent| {
        parent.spawn((TextSpan::new("World"), AccelerationText));
        parent.spawn((TextSpan::new("Chunk"), VelocityText));
    });
}

fn update_text(
    player: Query<&Rigidbody, With<Player>>,
    mut acceleration: Query<&mut TextSpan, With<AccelerationText>>,
    mut velocity: Query<&mut TextSpan, (With<VelocityText>, Without<AccelerationText>)>,
) {
    let player = player.single().unwrap();
    let mut acceleration = acceleration.single_mut().unwrap();
    let mut velocity = velocity.single_mut().unwrap();

    acceleration.0 = format!("Acceleration: {:.2}\n", player.acceleration);
    velocity.0 = format!("Velocity: {:.2}\n", player.velocity);
}
