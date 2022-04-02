let BITS_PER_BYTE: u32 = 8u;
let SIZEOF_U32: u32 = 4u;
let TILE_SIZE: u32 = 16u;
let CHUNK_SIZE: u32 = 16u;

struct Globals {
    resolution: vec2<u32>;
};

struct Locals {
    position: vec2<f32>;
    color_and_magnitude: u32;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] vertex_position: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(0), binding(1)]]
var<uniform> locals: Locals;
[[group(0), binding(2)]]
var unlit_texture: texture_2d<f32>;

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let vertex_index = i32(vertex_index);
    let resolution = vec2<f32>(globals.resolution);
    out.vertex_position = vec2<f32>(f32(vertex_index % 2 * 2 - 1), f32(vertex_index / 2 * 2 - 1));
    out.position = vec4<f32>((out.vertex_position + locals.position * 2.0) / resolution * f32(TILE_SIZE * CHUNK_SIZE), 0.0, 1.0);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let light_color_and_magnitude = unpack4x8unorm(locals.color_and_magnitude);
    let light_color = light_color_and_magnitude.rgb;
    let light_magnitude = light_color_and_magnitude.w;

    let dist = length(in.vertex_position) * f32(TILE_SIZE);
    let dist_sqr =  dist * dist;
    let chunk_size_sqr = f32(CHUNK_SIZE * CHUNK_SIZE);
    let normalized_distance = (-dist_sqr + chunk_size_sqr) / (chunk_size_sqr * (dist_sqr + 1.0));

    let raw_color = textureLoad(unlit_texture, vec2<i32>(in.position.xy), 0).rgb;
    let color = vec4<f32>(raw_color * light_color * normalized_distance, 1.0);
    return color;
}
