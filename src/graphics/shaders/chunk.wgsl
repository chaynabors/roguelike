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
    material: i32;
    primary_color: u32;
    secondary_color: u32;
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
var materials: texture_2d_array<f32>;

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let vertex_index = i32(vertex_index);
    let x = f32(vertex_index % 2 * 2 - 1);
    let y = f32(vertex_index / 2 * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

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

    let linear_sprite_offset = (up | down ^ (up | down) >> 1u) << 2u | (left | right ^ (left | right) >> 1u);
    let sprite_offset = vec2<i32>(vec2<u32>((linear_sprite_offset % 4u) * TILE_SIZE, (linear_sprite_offset / 4u) * TILE_SIZE));

    let tile_data = tile_data.data[tile];
    let material = tile_data.material;
    let primary_color = unpack4x8unorm(tile_data.primary_color);
    let secondary_color = unpack4x8unorm(tile_data.secondary_color);

    var color = textureLoad(materials, vec2<i32>(position.xy % TILE_SIZE) + sprite_offset, material, 0);
    color = primary_color * color + secondary_color * vec4<f32>(1.0 - color.rgb, color.a);
    return color;
}
