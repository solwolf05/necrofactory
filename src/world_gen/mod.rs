use bevy::prelude::*;
use bevy_modding::prelude::*;

use crate::{
    Player,
    world::{
        World, WorldPosition,
        chunk::{Chunk, TilePosition},
        tile::Tile,
    },
};

pub struct WorldGenPlugin;

impl Plugin for WorldGenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dynamic_gen);
    }
}

pub fn dev_gen(world: ResMut<World>) {
    let world = world.into_inner();
    for x in -1000..=1000 {
        for y in -1000..=1000 {
            let pos = IVec2::new(x, y);
            test_gen_chunk(world, pos);
        }
    }
}

pub fn dynamic_gen(world: ResMut<World>, player: Query<&Transform, With<Player>>) {
    let player_chunk = WorldPosition::from_bevy(player.single().unwrap().translation).chunk;
    let world = world.into_inner();
    for cy in -8..=8 {
        for cx in -8..=8 {
            let chunk_pos = player_chunk + IVec2::new(cx, cy);
            if !world.contains_chunk(chunk_pos) {
                test_gen_chunk(world, chunk_pos);
            }
        }
    }
}

pub fn test_gen_chunk(world: &mut World, pos: IVec2) {
    let mut chunk = Chunk::empty();
    for i in 0..=255 {
        if rand::random_bool(0.25) {
            chunk.insert(TilePosition::new(i), Tile::new(Id::new(1)));
        }
    }
    world.insert_chunk(pos, chunk);
}
