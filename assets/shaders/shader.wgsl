// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, //this is in frame buffer space (real window pixel coords such as 800x600)
    @location(0) vert_pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) tex_type: i32,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) details: vec4<i32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    // let x = f32(1 - i32(in_vertex_index)) * 0.5;
    // let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.vert_pos = out.clip_position.xyz;
    out.tex_type = instance.details[0];
    return out;
}

//
// fragment shader
//

// Texture bindings
@group(0) @binding(0)
var diffuse: binding_array<texture_2d<f32>>;
@group(0) @binding(1)
var diffuse_sampler: binding_array<sampler>;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var outval: vec4<f32>;
    var map = textureSample(diffuse[0], diffuse_sampler[0], in.tex_coords);
    var local = textureSample(diffuse[1], diffuse_sampler[1], in.tex_coords);
    var t = textureSample(diffuse[2], diffuse_sampler[2], in.tex_coords);
    var ct = textureSample(diffuse[3], diffuse_sampler[3], in.tex_coords);
    if in.tex_type == 0 {
        outval = map;
    } else if in.tex_type == 1 {
        outval = local;
    } else if in.tex_type == 2 {
        outval = t;
    } else {
        outval = ct;
    }
    return outval;
}