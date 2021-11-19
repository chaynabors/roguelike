[[block]]
struct Colors {
    colors: [[stride(4)]] array<u32>;
};

[[group(0), binding(0)]] var<storage, read_write> display: Colors;

[[stage(compute), workgroup_size(256)]]
fn main(
    [[builtin(workgroup_id)]] workgroup_id: vec3<u32>,
    [[builtin(local_invocation_index)]] local_invocation_index: u32,
) {
    let x = workgroup_id.x * u32(16) + local_invocation_index % u32(16);
    let y = (workgroup_id.y * u32(16) + local_invocation_index / u32(16)) * u32(832);
    display.colors[x + y] = u32(!0) << u32(u32(8) * workgroup_id.y) << u32(u32(8) * workgroup_id.x);
}
