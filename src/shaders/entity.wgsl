let BITS_PER_BYTE: u32 = 8u;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(1)]] tex_coords: vec2<f32>;
};

[[block]]
struct Globals {
    resolution: vec2<u32>;
    tile_size: u32;
    chunk_size: u32;
};

[[block]]
struct Locals {
    entity_position: vec2<u32>;
    entity_atlas_position: vec2<u32>;
    entity_color: u32;
    entity_detail: u32;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(0), binding(1)]]
var<uniform> locals: Locals;
[[group(0), binding(2)]]
var entity_atlas: texture_2d<f32>;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec2<f32>,
    [[location(1)]] tex_coords: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let position = (position + vec2<f32>(locals.entity_position)) / vec2<f32>(globals.resolution) * f32(globals.tile_size);
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.tex_coords = tex_coords;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let atlas_position = vec2<i32>((in.tex_coords + vec2<f32>(locals.entity_atlas_position)) * f32(globals.tile_size));
    let raw_color = textureLoad(entity_atlas, atlas_position, 0);
    var color = raw_color * unpack4x8unorm(locals.entity_color);
    color = color + vec4<f32>(1.0 - raw_color.rgb, raw_color.a) * unpack4x8unorm(locals.entity_detail);
    return vec4<f32>(color);
}
