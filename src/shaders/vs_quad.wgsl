[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let vertex_index = i32(vertex_index);
    let x = f32(vertex_index % 2 * 2 - 1);
    let y = f32(vertex_index / 2 * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}
