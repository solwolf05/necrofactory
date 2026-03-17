use crate::{
    debug::DebugText,
    physics::{Acceleration, Velocity},
    player::Player,
};
use bevy::prelude::*;

use crate::GameState;

pub struct PhysicsDebugPlugin;

impl Plugin for PhysicsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup.after(super::setup))
            .add_systems(Update, update_text.run_if(in_state(GameState::InGame)));
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
    player: Query<(&Velocity, &Acceleration), With<Player>>,
    mut acceleration_text: Query<&mut TextSpan, With<AccelerationText>>,
    mut velocity_text: Query<&mut TextSpan, (With<VelocityText>, Without<AccelerationText>)>,
) {
    let (velocity, acceleration) = player.single().unwrap();
    let mut acceleration_text = acceleration_text.single_mut().unwrap();
    let mut velocity_text = velocity_text.single_mut().unwrap();

    acceleration_text.0 = format!("Acceleration: {:.2}\n", acceleration.0);
    velocity_text.0 = format!("Velocity: {:.2}\n", velocity.0);
}
