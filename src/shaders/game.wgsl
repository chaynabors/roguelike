[[block]]
struct Uniforms {
    resolution: vec2<f32>;
    map_width: i32;
    map_height: i32;
    sprite_size: i32;
};

struct TileData {
    atlas_position: vec2<i32>;
    color: u32;
    detail: u32;
};

[[block]]
struct TileAtlas {
    data: array<TileData, 256>;
};

[[block]]
struct SceneLayout {
    data: [[stride(4)]] array<u32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;
[[group(0), binding(1)]]
var sprite_atlas: texture_2d<f32>;
[[group(0), binding(2)]]
var<storage, read> tile_atlas: TileAtlas;
[[group(0), binding(3)]]
var<storage, read> scene_layout: SceneLayout;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec2<f32>,
) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

[[stage(fragment)]]
fn draw_map(
    [[builtin(position)]] position: vec4<f32>,
) -> [[location(0)]] vec4<f32> {
    let iposition = vec2<i32>(position.xy);
    let sprite_size = uniforms.sprite_size;

    let tile_index = iposition.x / sprite_size % uniforms.map_width + iposition.y / sprite_size % uniforms.map_height * uniforms.map_width;
    var tile = scene_layout.data[tile_index / 4];
    tile = tile >> (u32(tile_index) % 4u * 8u) & 0xffu;

    let tile_data = tile_atlas.data[tile];
    let tile_position = tile_data.atlas_position;
    let tile_color = unpack4x8unorm(tile_data.color);
    let tile_detail = unpack4x8unorm(tile_data.detail);

    let sprite_atlas_dimensions = textureDimensions(sprite_atlas);
    let raw_color = textureLoad(sprite_atlas, (iposition % vec2<i32>(sprite_size) + tile_position * sprite_size) % sprite_atlas_dimensions, 0);
    var color = raw_color * tile_color;
    color = color + (vec4<f32>(1.0, 1.0, 1.0, raw_color.a) - raw_color) * tile_detail;
    return color;
}

[[group(0), binding(0)]]
var unlit_map: texture_2d<f32>;

[[stage(fragment)]]
fn draw_light(
    [[builtin(position)]] position: vec4<f32>,
) -> [[location(0)]] vec4<f32> {
    let distance = distance(vec2<f32>(3.5, 4.5), position.xy / 16.0);
    let squared_distance = distance * distance;
    let inverse_squared_distance = min(1.0 / squared_distance, 1.0);

    let light_color = vec4<f32>(0.8, 0.74, 0.7, 1.0);

    let light = light_color * inverse_squared_distance;

    let iposition = vec2<i32>(position.xy);
    let unlit_map_dimensions = textureDimensions(unlit_map);
    return textureLoad(unlit_map, iposition % unlit_map_dimensions, 0) * light;
}
