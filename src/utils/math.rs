// Some math stuff for viewmatricies and such

use crate::datatypes::{tmp_vec3, tmp_vec2};

/*
vec2_t utilities::world_to_screen(vec3_t world_position)
{
    vec2_t result;
    float _x = view_matrix[0][0] * world_position.x + view_matrix[0][1] * world_position.y + view_matrix[0][2] * world_position.z + view_matrix[0][3];
    float _y = view_matrix[1][0] * world_position.x + view_matrix[1][1] * world_position.y + view_matrix[1][2] * world_position.z + view_matrix[1][3];
    float w = view_matrix[3][0] * world_position.x + view_matrix[3][1] * world_position.y + view_matrix[3][2] * world_position.z + view_matrix[3][3];

    if (w < 0.01f)
        return vec2_t{ 0, 0 };

    float inv_w = 1.f / w;
    _x *= inv_w;
    _y *= inv_w;

    result.x = res_x * .5f;
    result.y = res_y * .5f;

    result.x += 0.5f * _x * res_x + 0.5f;
    result.y -= 0.5f * _y * res_y + 0.5f;

    return result;
}

*/

/* 
// This is the world to screen function for use with the built in game view matrix. Perfectly Functional. Uncomment for use
pub fn world_2_screen(world_pos: &tmp_vec3, view_matrix: &[[f32;4];4], screen_width: Option<f32>, screen_height: Option<f32>) -> Option<tmp_vec3> {
    let mut _x:f32 = view_matrix[0][0] * world_pos.x + view_matrix[0][1] * world_pos.y + view_matrix[0][2] * world_pos.z + view_matrix[0][3];
    let mut _y:f32 = view_matrix[1][0] * world_pos.x + view_matrix[1][1] * world_pos.y + view_matrix[1][2] * world_pos.z + view_matrix[1][3];
    let w:f32 = view_matrix[3][0] * world_pos.x + view_matrix[3][1] * world_pos.y + view_matrix[3][2] * world_pos.z + view_matrix[3][3];
    if w < 0.8 {
        None
    } else {
        let inverse_w = 1. / w;
        _x *= inverse_w;
        _y *= inverse_w;
        let res_x = screen_width.unwrap_or(1920.);
        let res_y = screen_height.unwrap_or(1080.);
        Some(tmp_vec3{
            x: (res_x * 0.5) + 0.5 * _x * res_x + 0.5,
            y: (res_y * 0.5) - 0.5 * _y * res_y + 0.5,
            z: inverse_w
        })
    }
}

*/

pub fn angle_to_vec(x:f32, y:f32) -> tmp_vec3 {
    rad_to_vec(d2r(x), d2r(y))
}

pub fn rad_to_vec(x:f32,y:f32) -> tmp_vec3{
    tmp_vec3 {
        x: f32::cos(x) * f32::cos(y),
        y: f32::cos(x) * f32::sin(y),
        z: -f32::sin(x)
    }
}

pub fn d2r(d:f32)->f32{
    (d as f64*(std::f64::consts::PI/180.)) as f32
}

/// this is used by recoil recorder and other stuff to get a world point a distance in from the screen center (at look direction)
#[allow(dead_code)]
pub fn get_crosshair_world_point_at_dist(to_dist: f32, our_pos: tmp_vec3, eye_ang: tmp_vec3) -> tmp_vec3 {
    // get direction vector for our view angles
    let eye_vec = angle_to_vec(eye_ang.x, eye_ang.y);
    // now that we have a direction vector (unit) and a magnitude
    // we can get the point along our look direction line with origin + dist*unit
    our_pos + eye_vec*to_dist
}

pub fn get_dist_from_crosshair(to_pos: tmp_vec3, our_pos: tmp_vec3, eye_ang: tmp_vec2) -> f32 {
    // difference
    let diff = to_pos - our_pos;
    // get direction vector for our view angles
    let eye_vec = angle_to_vec(eye_ang.x, eye_ang.y);
    // get the magnitide (distance) between to and from
    let dmag = diff.magnitude();

    // now that we have a direction vector (unit) and a magnitude
    // we can get the point along our look direction line with origin + dist*unit
    let point = our_pos + eye_vec*dmag;

    // now get the distance from this new point to the target point
    let diff2 = to_pos - point;
    diff2.magnitude()
}

pub fn round_up(num_in: u64, up_to_multiple: u64) -> u64 {
    if up_to_multiple <=0 {return num_in}
    let remainder = num_in % up_to_multiple;
    if (remainder == 0) {return num_in}
    num_in + up_to_multiple - remainder
}

pub fn radar_scale(x:f32,y:f32,scale:f32, map_x:f32, map_y:f32, window_size:Option<(f32,f32)>) -> (f32,f32) {
 let mut nx = x - map_x;
 let mut ny = y - map_y;

 nx = nx / scale;
 ny = ny / scale;

 // now divid map by width and height of map in px
 nx = nx / 1024.0;
 ny = ny / 1024.0;

 // invert y
 //ny = ny * -1.0;

 if let Some((winx,winy)) = window_size {
     // scale it to either window width or height depending on which is smaller
    if winx < winy {
        nx = nx * winx;
        ny = ny * winx;
    } else {
        nx = nx * winy;
        ny = ny * winy;
    }
 }

(nx,ny)
}