struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[block]]
struct Globals {
    resolution: vec2<u32>;
    tile_size: u32;
    chunk_size: u32;
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
    [[location(1)]] tex_coords: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.tex_coords = tex_coords;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let light_position = vec2<f32>(globals.resolution) / 2.0 - f32(globals.tile_size) / 2.0 + locals.position * f32(globals.tile_size * globals.chunk_size);
    let light_color = unpack4x8unorm(locals.color).rgb;
    var light_magnitude = locals.magnitude;

    let dist = distance(light_position, in.position.xy) / f32(globals.tile_size);
    let dist_sqr =  dist * dist;
    let chunk_size_sqr = f32(globals.chunk_size * globals.chunk_size);
    //let inverse_squared_distance = 1.0 / (1.0 + dist * dist);
    let normalized_distance = (-dist_sqr + chunk_size_sqr) / (chunk_size_sqr * (dist_sqr + 1.0));

    let raw_color = textureLoad(unlit_texture, vec2<i32>(in.position.xy), 0).rgb;
    let color = vec4<f32>(raw_color * light_color * normalized_distance, 1.0);
    return color;
}
