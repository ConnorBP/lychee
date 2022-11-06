// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, //this is in frame buffer space (real window pixel coords such as 800x600)
    @location(0) vert_pos: vec3<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    // let x = f32(1 - i32(in_vertex_index)) * 0.5;
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.vert_pos = model.position;
    return out;
}

// fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color,1.0);
}