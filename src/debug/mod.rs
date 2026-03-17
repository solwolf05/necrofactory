use bevy::prelude::*;

use crate::GameState;

pub mod coord;
pub mod physics;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup)
            .add_systems(OnExit(GameState::InGame), cleanup);
    }
}

#[derive(Component)]
pub struct DebugText;

fn setup(mut commands: Commands) {
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        },
        DebugText,
    ));
}

fn cleanup(mut commands: Commands, text: Query<Entity, With<DebugText>>) {
    commands.entity(text.single().unwrap()).despawn();
}
