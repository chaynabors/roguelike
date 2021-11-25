[[block]]
struct Globals {
    resolution: vec2<f32>;
    tile_size: vec2<f32>;
    chunk_size: vec2<f32>;
};

[[block]]
struct Locals {
    position: vec2<f32>;
    color: u32;
    magnitude: f32;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(0), binding(1)]]
var<uniform> locals: Locals;
[[group(0), binding(2)]]
var unlit_texture: texture_2d<f32>;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec2<f32>,
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main(
    [[builtin(position)]] position: vec4<f32>,
) -> [[location(0)]] vec4<f32> {
    let light_position = locals.position * globals.tile_size;
    let light_color = unpack4x8unorm(locals.color).rgb;
    var light_magnitude = locals.magnitude;

    let dist = distance(light_position, position.xy) / 16.0;
    let inverse_squared_distance = min(1.0 / (dist * dist), 1.0);

    let unlit_texture_dimensions = textureDimensions(unlit_texture);
    let raw_color = textureLoad(unlit_texture, vec2<i32>(position.xy) % unlit_texture_dimensions, 0).rgb;
    let color = vec4<f32>(raw_color * light_color * inverse_squared_distance, 1.0);
    return color;
}
