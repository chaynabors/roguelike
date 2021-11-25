let BITS_PER_BYTE: u32 = 8u;

struct TileData {
    atlas_position: vec2<f32>;
    color: f32;
    detail: f32;
};

[[block]]
struct Globals {
    resolution: vec2<f32>;
    tile_size: vec2<f32>;
    chunk_size: vec2<f32>;
};

[[block]]
struct Locals {
    chunk_position: vec2<f32>;
    chunk_layout: array<f32, 64>;
};

[[block]]
struct TileDataAtlas {
    data: array<TileData, 256>;
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
fn vs_main(
    [[location(0)]] position: vec2<f32>,
) -> [[builtin(position)]] vec4<f32> {
    let pixels_per_chunk = globals.tile_size * globals.chunk_size;
    let chunks_per_screen = globals.resolution / pixels_per_chunk;
    return vec4<f32>(position / globals.resolution * pixels_per_chunk + (locals.chunk_position / chunks_per_screen + 0.5), 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main(
    [[builtin(position)]] position: vec4<f32>,
) -> [[location(0)]] vec4<f32> {
    let tiles_per_byte = globals.chunk_size.x * globals.chunk_size.y / arrayLength(locals.chunk_layout);
    let tile_index = position.xy / globals.tile_size % globals.chunk_size;
    var tile = locals.chunk_layout[i32(tile_index.x + tile_index.y * globals.chunk_size.x / tiles_per_byte)];
    tile = tile >> i32(tile_index % locals.tiles_per_byte) * BITS_PER_BYTE & 0xff;

    let tile_data = tile_data_atlas.data[tile];
    let tile_position = tile_data.atlas_position * globals.tile_size;
    let tile_color = unpack4x8unorm(tile_data.color);
    let tile_detail = unpack4x8unorm(tile_data.detail);

    let raw_color = textureLoad(tile_atlas, vec2<i32>(position % globals.tile_size + tile_position), 0);
    var color = raw_color * tile_color;
    color = color + vec4<f32>(1.0 - raw_color.rgb, raw_color.a) * tile_detail;
    return color;
}
