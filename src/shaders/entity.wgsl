let BITS_PER_BYTE: u32 = 8u;

[[block]]
struct Globals {
    resolution: vec2<f32>;
    tile_size: vec2<f32>;
    chunk_size: vec2<f32>;
};

[[block]]
struct Locals {
    entity_position: vec2<f32>;
    entity_atlas_position: vec2<f32>;
    entity_color: f32,
    entity_detail: f32,
};

[[block]]
struct TileDataAtlas {
    data: array<EntityData, 256>;
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
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position / globals.resolution * globals.tile_size + locals.entity_position * globals.tile_size, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main(
    [[builtin(position)]] position: vec4<f32>,
) -> [[location(0)]] vec4<f32> {
    let entity_atlas_position = locals.entity_atlas_position * globals.tile_size;
    let raw_color = textureLoad(entity_atlas, vec2<i32>(position % globals.tile_size + entity_atlas_position), 0);
    var color = raw_color * unpack4x8unorm(locals.entity_color);
    color = color + vec4<f32>(1.0 - raw_color.rgb, raw_color.a) * unpack4x8unorm(locals.entity_detail);
    return color;
}
