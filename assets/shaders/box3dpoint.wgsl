
// from https://iquilezles.org/articles/distfunctions
// https://www.shadertoy.com/view/7tsXRN
// https://stackoverflow.com/questions/68233304/how-to-create-a-proper-rounded-rectangle-in-webgl
fn rounded_box_sdf(center_pos: vec2<f32>, box_size: vec2<f32>, radius: f32) -> f32
{
    return length(max(abs(center_pos) - box_size + radius, vec2(0.0))) - radius;
}

// Convert from clip space coordinates (-1 to +1) into screen coordinate space (e.x. from 0-1920)
fn clip_to_screen(clip_pos: vec2<f32>,screen_size: vec2<f32>) -> vec2<f32> {
    return (clip_pos + 1.0) * 0.5 * screen_size;
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct RotationUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(1)
var<uniform> rotation: RotationUniform;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, //this is in frame buffer space (real window pixel coords such as 800x600)
    @location(0) vert_pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    // model: VertexInput,
    instance: InstanceInput
) -> VertexOutput  {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    // const vertices of a quad
    var QUAD_VERTS: array<vec4<f32>,4>  = array(
        vec4(-0.5, -0.5, 0.0, 1.0),
        vec4(0.5, -0.5, 0.0, 1.0),
        vec4(0.5, 0.5, 0.0, 1.0),
        vec4(-0.5, 0.5, 0.0, 1.0)
    );
    // const text coordinate mapping for the quad
    var QUAD_TEX_COORDS: array<vec2<f32>,4> = array(
        vec2(0.0, 1.0),
        vec2(1.0, 1.0),
        vec2(1.0, 0.0),
        vec2(0.0, 0.0)
    );

    //out.clip_position = camera.view * camera.proj * model_matrix * QUAD_VERTS[in_vertex_index];

    let vertex =  rotation.view_proj * QUAD_VERTS[in_vertex_index];
    let vertex_position = vec4<f32>(-vertex.x, vertex.y, vertex.z, 1.0);
    let position =  camera.view_proj * model_matrix * vertex_position;

    // let camera_right = normalize(vec3<f32>(camera.view_proj.x.x,camera.view_proj.y.x,camera.view_proj.z.x));
    // let camera_up = vec3<f32>(0.0,1.0,0.0);
    // let world_space = camera_right * vertex_position.x + camera_up * vertex_position.y;
    // let position = camera.view_proj * model_matrix * vec4<f32>(world_space, 1.0);

    out.clip_position = position;

    out.tex_coords = QUAD_TEX_COORDS[in_vertex_index];
    out.vert_pos = out.clip_position.xyz;
    out.color = instance.color;

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    let  size           = vec2<f32>(1.0, 1.0);
    let thickness      = 0.1;
    let shadowSoftness = 0.1;
    let  shadowOffset   = vec2<f32>(0.1, -0.1);
    let edgeSoftness   = 0.001;

    // radius from 0 to 1 (percentage)
    let radius         = 0.3;
    
    let distance: f32       = rounded_box_sdf((1.0 + thickness) * (in.tex_coords - 0.5), size/2.0, radius);
    let smoothedAlpha  = 1.0 - smoothstep(-edgeSoftness, edgeSoftness, abs(distance) - thickness);
    if(smoothedAlpha < 0.01) {
        discard;
    }
    // let bg = vec4(in.tex_coords, 1.0, 1.0);
    let bg = vec4(0.0);
    let  quadColor      = mix(bg, in.color, smoothedAlpha);
    
    let shadowDistance = rounded_box_sdf(((1.0 + thickness) * (shadowOffset - in.tex_coords - 0.5)), size/2.0, radius);
    let shadowAlpha    = 1.0 - smoothstep(-shadowSoftness/2.0, shadowSoftness/2.0, abs(shadowDistance));
    let shadowColor     = vec4(0.8, 0.0, 0.8, 1.0);
    return mix(quadColor, shadowColor, shadowAlpha - smoothedAlpha);
    //return vec4<f32>(in.vert_pos.x,in.vert_pos.y,distance,1.0);
    // return vec4<f32>(0.3,0.2,0.1,1.0);
}