#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var tile_texture: texture_2d<f32>;
@group(2) @binding(1) var tile_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tile_texture, tile_sampler, in.uv);
}
