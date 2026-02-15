use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    modding::{Id, Registry, TileHandles},
    world::tile::TileDef,
};

#[derive(Debug, Resource)]
pub struct TextureAtlasMap {
    pub map: HashMap<Id<TileDef>, usize>,
    pub layout: TextureAtlasLayout,
    pub sources: TextureAtlasSources,
    pub texture: Handle<Image>,
}

pub fn build_texture_atlas(
    mut commands: Commands,
    tiles: Res<Registry<TileDef>>,
    handles: Res<TileHandles>,
    mut textures: ResMut<Assets<Image>>,
) {
    let mut builder = TextureAtlasBuilder::default();

    builder
        .max_size(UVec2::splat(4608))
        .padding(UVec2::splat(2));

    for (&_id, handle) in handles.complete.iter() {
        let texture = textures.get(handle).unwrap();
        builder.add_texture(Some(handle.id()), texture);
    }

    let (layout, sources, image) = builder.build().unwrap();
    let texture = textures.add(image);

    let mut map = HashMap::new();
    for (id, ..) in tiles.iter_with_id() {
        let texture = handles.complete.get(&id).unwrap();
        map.insert(id, sources.texture_index(texture).unwrap());
    }

    commands.insert_resource(TextureAtlasMap {
        map,
        layout,
        sources,
        texture,
    });
}
