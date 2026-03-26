use crate::{
    debug::DebugText,
    input::WorldCursor,
    modding::Registry,
    world::{World, tile::TileDef},
};
use bevy::prelude::*;

use crate::GameState;

pub struct ProbePlugin;

impl Plugin for ProbePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), setup.after(super::setup))
            .add_systems(Update, update_text.run_if(in_state(GameState::InGame)));
    }
}

#[derive(Component)]
pub struct ProbeText;

fn setup(mut commands: Commands, text_root: Query<Entity, With<DebugText>>) {
    let text_root = text_root.single().unwrap();
    commands.entity(text_root).with_children(|parent| {
        parent.spawn((TextSpan::new("Probe"), ProbeText));
    });
}

fn update_text(
    cursor: Res<WorldCursor>,
    world: Res<World>,
    registry: Res<Registry<TileDef>>,
    mut text: Query<&mut TextSpan, With<ProbeText>>,
) {
    let mut text = text.single_mut().unwrap();

    let path = cursor
        .0
        .and_then(|cursor| world.get_tile(cursor.into()))
        .and_then(|tile| registry.resolve(tile.id));

    text.0 = match path {
        Some(path) => format!("Tile: {}\n", path),
        None => "Tile: none\n".to_owned(),
    };
}
