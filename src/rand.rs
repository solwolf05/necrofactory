use bevy::prelude::*;
use noiz::{prelude::*, rng::NoiseRng};

#[derive(Debug, Default)]
pub struct RandPlugin {
    pub seed: u32,
}

impl RandPlugin {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }
}

impl Plugin for RandPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(MasterSeed(self.seed));
        app.add_systems(Startup, spawn_rngs);
    }
}

#[derive(Debug, Resource)]
pub struct MasterSeed(pub u32);

#[derive(Resource)]
pub struct WorldHeightRng(pub NoiseRng, pub Random<UNorm, f32>);

fn spawn_rngs(mut commands: Commands, seed: Res<MasterSeed>) {
    commands.insert_resource(WorldHeightRng(NoiseRng(seed.0), Random::default()));
}
