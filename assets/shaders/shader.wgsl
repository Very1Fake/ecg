// Vertex Shader

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
}

// This function is used to transform vertices
@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Some transformation
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;

    out.clip_pos = vec4<f32>(x, y, 0.0, 1.0);

    return out;
}

// Fragments are pixels, and function is used to color them
@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // Just contant color for every fragment
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
