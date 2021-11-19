[[block]]
struct Parameters {
    width: u32;
    height: u32;
};

[[block]]
struct Colors {
    colors: [[stride(4)]] array<u32>;
};

[[group(0), binding(0)]] var<uniform> params: Parameters; 
[[group(0), binding(1)]] var<storage, read_write> display: Colors;

[[stage(compute), workgroup_size(16u, 16u)]]
fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
    display.colors[global_invocation_id.x + global_invocation_id.y * params.width] = 0xffffffffu << global_invocation_id.x << global_invocation_id.y;
}
