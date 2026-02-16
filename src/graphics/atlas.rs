use std::collections::HashMap;

use bevy::{
    prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::Material2d,
};

use crate::{
    modding::{Id, Registry, TileHandles},
    world::tile::TileDef,
};

#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct TilemapMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material2d for TilemapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tilemap.wgsl".into()
    }
}

#[derive(Debug, Resource)]
pub struct TextureAtlasMap {
    map: HashMap<Id<TileDef>, usize>,
    layout: TextureAtlasLayout,
    sources: TextureAtlasSources,
    pub texture: Handle<Image>,
    pub material: Handle<TilemapMaterial>,
}

impl TextureAtlasMap {
    pub fn rect(&self, id: Id<TileDef>) -> Option<Rect> {
        let index = *self.map.get(&id)?;
        let urect = self.layout.textures[index];
        let size = self.layout.size;
        // let texel = 0.5 / size;

        Some(Rect {
            min: Vec2::new(
                (urect.min.x as f32 + 0.05) / size.x as f32,
                (urect.min.y as f32 + 0.05) / size.y as f32,
            ),
            max: Vec2::new(
                (urect.max.x as f32 - 0.05) / size.x as f32,
                (urect.max.y as f32 - 0.05) / size.y as f32,
            ),
        })
    }
}

pub fn build_texture_atlas(
    mut commands: Commands,
    tiles: Res<Registry<TileDef>>,
    handles: Res<TileHandles>,
    mut textures: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
) {
    let mut builder = TextureAtlasBuilder::default();

    builder
        // .max_size(UVec2::splat(4608))
        .padding(UVec2::splat(2));

    for (&_id, handle) in handles.complete.iter() {
        let texture = textures.get(handle).unwrap();
        builder.add_texture(Some(handle.id()), texture);
    }

    let (layout, sources, mut image) = builder.build().unwrap();
    image.sampler = bevy::image::ImageSampler::nearest();
    image.texture_descriptor.usage = bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
        | bevy::render::render_resource::TextureUsages::COPY_DST;

    let texture = textures.add(image);

    let mut map = HashMap::new();
    for (id, ..) in tiles.iter_with_id() {
        let texture = handles.complete.get(&id).unwrap();
        map.insert(id, sources.texture_index(texture).unwrap());
    }

    let material = TilemapMaterial {
        texture: texture.clone(),
    };
    let material = materials.add(material);

    commands.insert_resource(TextureAtlasMap {
        map,
        layout,
        sources,
        texture,
        material,
    });
}
