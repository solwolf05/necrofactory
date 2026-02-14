use bevy::prelude::*;

pub fn build_texture_atlas() {
    let builder = TextureAtlasBuilder::default()
        .max_size(UVec2::splat(4608))
        .padding(UVec2::splat(2));
}
