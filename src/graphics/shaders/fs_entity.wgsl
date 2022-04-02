let TILE_SIZE: u32 = 16u;

struct Locals {
    position: vec2<f32>;
    atlas_position: vec2<u32>;
    size: vec2<u32>;
    color: u32;
    detail: u32;
};

[[group(0), binding(0)]]
var<uniform> locals: Locals;
[[group(0), binding(1)]]
var entity_atlas: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let tile_position = vec2<i32>(locals.atlas_position * TILE_SIZE);
    let pixel_position = vec2<i32>(position.xy % f32(TILE_SIZE));
    let mono_color = textureLoad(entity_atlas, tile_position + pixel_position, 0);

    let color = unpack4x8unorm(locals.color).rgb;
    let detail = unpack4x8unorm(locals.detail).rgb;

    let out = mono_color.rgb * color;
    let out = vec4<f32>(out + (1.0 - mono_color.rgb) * detail, mono_color.a);
    return out;
}
