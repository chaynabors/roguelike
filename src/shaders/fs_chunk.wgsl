let BITS_PER_BYTE: u32 = 8u;
let SIZEOF_U32: u32 = 4u;
let TILE_SIZE: u32 = 16u;
let CHUNK_SIZE: u32 = 16u;

struct Globals {
    resolution: vec2<u32>;
};

struct Locals {
    chunk_position: vec2<i32>;
};

struct ChunkData {
    data: array<u32>;
};

struct Tile {
    atlas_position: vec2<u32>;
    color: u32;
    detail: u32;
};

struct TileData {
    data: array<Tile, 256u>;
};

[[group(0), binding(0)]]
var<uniform> globals: Globals;
[[group(0), binding(1)]]
var<uniform> locals: Locals;
[[group(0), binding(2)]]
var<storage, read> chunk_data: ChunkData;
[[group(0), binding(3)]]
var<storage, read> tile_data: TileData;
[[group(0), binding(4)]]
var tile_atlas: texture_2d<f32>;

fn get_tile(x: u32, y: u32) -> u32 {
    let pixels_per_chunk_axis = TILE_SIZE * CHUNK_SIZE;
    let tiles_per_chunk = CHUNK_SIZE * CHUNK_SIZE;
    let chunks_per_row = u32(ceil(f32(globals.resolution.x) / f32(pixels_per_chunk_axis) + 3.0));
    let x = x / TILE_SIZE % TILE_SIZE + x / pixels_per_chunk_axis * tiles_per_chunk;
    let y = y / TILE_SIZE % TILE_SIZE * CHUNK_SIZE + y / pixels_per_chunk_axis * tiles_per_chunk * chunks_per_row;
    let tile = chunk_data.data[(x + y) / 4u];
    return (tile >> ((x % 4u) * BITS_PER_BYTE)) & 0xffu;
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let resolution = globals.resolution;
    let buffer_size = arrayLength(&chunk_data.data);
    let position = vec2<u32>(position.xy);

    let tile = get_tile(position.x, position.y);
    let up = u32(tile == get_tile(position.x, position.y - TILE_SIZE)) << 1u;
    let down = u32(tile == get_tile(position.x, position.y + TILE_SIZE));
    let left = u32(tile == get_tile(position.x - TILE_SIZE, position.y)) << 1u;
    let right = u32(tile == get_tile(position.x + TILE_SIZE, position.y));

    var linear_sprite_offset = (up | down ^ (up | down) >> 1u) << 2u | (left | right ^ (left | right) >> 1u);
    let sprite_offset = vec2<u32>(linear_sprite_offset % 4u, linear_sprite_offset / 4u) * TILE_SIZE;

    let tile_data = tile_data.data[tile];
    let atlas_position = vec2<i32>(tile_data.atlas_position * vec2<u32>(TILE_SIZE * 4u) + sprite_offset);
    let tile_color = unpack4x8unorm(tile_data.color);
    let tile_detail = unpack4x8unorm(tile_data.detail);

    let raw_color = textureLoad(tile_atlas, vec2<i32>(position.xy % TILE_SIZE) + atlas_position, 0);
    let color = raw_color * tile_color + vec4<f32>(1.0 - raw_color.rgb, raw_color.a) * tile_detail;
    let debug = vec4<f32>(f32(linear_sprite_offset));
    return color;
}
