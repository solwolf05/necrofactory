use crate::{
    debug::DebugText,
    player::{FuelTank, Jetpack, JetpackControl, Player},
};
use bevy::prelude::*;

use crate::GameState;

pub struct JetPackDebugPlugin;

impl Plugin for JetPackDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup.after(super::setup))
            .add_systems(Update, update_text.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct JetPackFuelText;

fn setup(mut commands: Commands, text_root: Query<Entity, With<DebugText>>) {
    let text_root = text_root.single().unwrap();
    commands.entity(text_root).with_children(|parent| {
        parent.spawn((TextSpan::new("JetPack Fuel"), JetPackFuelText));
    });
}

fn update_text(
    player: Query<(&FuelTank, &JetpackControl), With<Player>>,
    mut text: Query<&mut TextSpan, With<JetPackFuelText>>,
) {
    let (fuel_tank, control) = player.single().unwrap();
    let mut text = text.single_mut().unwrap();

    let fuel = (fuel_tank.fuel / fuel_tank.max_fuel * 10.0).floor() as usize;

    text.0 = format!(
        "Jetpack: {} {} {}\n",
        control.throttle,
        if control.hover { "H" } else { "h" },
        "|".repeat(fuel)
    );
}
