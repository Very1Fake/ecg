/// Camera

struct CameraUniform {
    projection: mat4x4<f32>,
}

@group(0)
@binding(0)
var<uniform> camera: CameraUniform;


/// Vertex Shader

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// This function is used to transform vertices
@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    // Reassembling of model matrix
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // Manual casting of `VertexModel` to `VertexOutput`
    out.clip_pos = camera.projection * model_matrix * vec4<f32>(model.pos, 1.0);
    out.color = model.color;

    return out;
}


/// Fragment shader

// Fragments are pixels, and function is used to color them
@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // Just contant color for every fragment
    return vec4<f32>(in.color, 1.0);
}
