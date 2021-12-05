let BITS_PER_BYTE: u32 = 8u;

struct TileData {
    atlas_position: vec2<u32>;
    color: u32;
    detail: u32;
};

[[block]]
struct Globals {
    resolution: vec2<u32>;
    tile_size: u32;
    chunk_size: u32;
};

[[block]]
struct Locals {
    chunk_position: vec2<i32>;
    [[align(16)]] chunk_layout: [[stride(16)]] array<u32, 64u>;
};

[[block]]
struct TileDataAtlas {
    data: array<TileData, 256u>;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(0), binding(1)]]
var<uniform> locals: Locals;
[[group(0), binding(2)]]
var<storage, read> tile_data_atlas: TileDataAtlas;
[[group(0), binding(3)]]
var tile_atlas: texture_2d<f32>;

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let vertex_index = i32(vertex_index);
    let x = f32(vertex_index % 2 * 2 - 1);
    let y = f32(vertex_index / 2 * -2 + 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let tile_index = vec2<u32>(in.tex_coords);
    let linear_tile_index = tile_index.x + tile_index.y * globals.chunk_size;
    var tile = locals.chunk_layout[linear_tile_index / 4u];
    tile = (tile >> (24u - linear_tile_index * BITS_PER_BYTE)) & 0xffu;

    let tile_data = tile_data_atlas.data[tile];
    let tile_position = tile_data.atlas_position;
    let tile_color = unpack4x8unorm(tile_data.color);
    let tile_detail = unpack4x8unorm(tile_data.detail);

    let pixels_per_chunk = globals.tile_size * globals.chunk_size;
    let atlas_position = vec2<i32>((in.tex_coords % 1.0 + vec2<f32>(tile_position)) * f32(globals.tile_size));
    let raw_color = textureLoad(tile_atlas, atlas_position, 0);
    var color = raw_color * tile_color;
    color = color + vec4<f32>(1.0 - raw_color.rgb, 1.0) * tile_detail;
    return color;
}
