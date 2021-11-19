[[block]]
struct Parameters {
    tile_size: u32;
    width: u32;
    height: u32;
};

[[block]]
struct Colors {
    colors: [[stride(4)]] array<u32>;
};

[[group(0), binding(0)]] var<uniform> params: Parameters; 
[[group(0), binding(1)]] var<storage, read_write> display: Colors;

[[stage(compute), workgroup_size(256)]]
fn main(
    [[builtin(workgroup_id)]] workgroup_id: vec3<u32>,
    [[builtin(local_invocation_index)]] local_invocation_index: u32,
) {
    let x = workgroup_id.x * params.tile_size + local_invocation_index % params.tile_size;
    let y = (workgroup_id.y * params.tile_size + local_invocation_index / params.tile_size) * params.width;
    display.colors[x + y] = !0u << (8u * workgroup_id.y) << (8u * workgroup_id.x);
}
