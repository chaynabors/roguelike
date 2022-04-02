let BITS_PER_BYTE: u32 = 8u;
let SIZEOF_U32: u32 = 4u;
let TILE_SIZE: u32 = 16u;
let CHUNK_SIZE: u32 = 16u;

struct Globals {
    resolution: vec2<u32>;
};

struct Locals {
    tile: u32;
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
[[group(0), binding(3)]]
var<storage, read> tile_data: TileData;
[[group(0), binding(4)]]
var tile_atlas: texture_2d<f32>;

[[stage(fragment)]]
fn fs_main([[builtin(position)]] position: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let tile_data = tile_data.data[tile];
    let atlas_position = vec2<i32>(tile_data.atlas_position * vec2<u32>(TILE_SIZE * 4u) + sprite_offset);
    let tile_color = unpack4x8unorm(tile_data.color);
    let tile_detail = unpack4x8unorm(tile_data.detail);

    let raw_color = textureLoad(tile_atlas, vec2<i32>(position.xy % TILE_SIZE) + atlas_position, 0);
    let color = raw_color * tile_color + vec4<f32>(1.0 - raw_color.rgb, raw_color.a) * tile_detail;
    let debug = vec4<f32>(f32(linear_sprite_offset));
    return color;
}
