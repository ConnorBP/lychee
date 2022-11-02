// Some math stuff for viewmatricies and such

use lazy_static::lazy_static;
use log::info;

use crate::gamedata::entitylist::tmp_vec3;

lazy_static! {
    /*
#define FORWARD_DIRECTION_UNIT glm::vec3(1.0f, 0.0f, 0.0f)
#define RIGHT_DIRECTION_UNIT glm::vec3(0.0f, -1.0f, 0.0f)
#define UP_DIRECTION_UNIT glm::vec3(0.0f, 0.0f, 1.0f)
    */
    static ref FORWARD_DIRECTION_UNIT: glm::Vec3 = glm::vec3(1.0, 0.0, 0.0);
    static ref RIGHT_DIRECTION_UNIT: glm::Vec3 = glm::vec3(0.0, -1.0, 0.0);
    static ref UP_DIRECTION_UNIT: glm::Vec3 = glm::vec3(0.0, 0.0, 1.0);
}

/*
namespace math
{
	glm::mat4 createProjectionViewMatrix(const glm::vec3& cameraPosition, const glm::vec3& cameraEulerAngles, float aspectRatio = 16.0f / 9.0f, float fieldOfViewYDegrees = 75.0f, float zNear = 1.f, float zFar = 4000.0f);

	glm::mat4 createProjectionViewMatrix(const glm::vec3& cameraPosition, const glm::quat& cameraRotation, float aspectRatio = 16.0f / 9.0f, float fieldOfViewYDegrees = 75.0f, float zNear = 1.f, float zFar = 4000.0f);

	glm::vec2 transformWorldPointIntoScreenSpace(const glm::vec3& worldPoint, const glm::mat4& projectionViewMatrix, float screenWidth, float screenHeight);
}

*/

//const glm::vec3& cameraPosition, const glm::vec3& cameraEulerAngles, float aspectRatio, float fieldOfViewYDegrees, float zNear, float zFar
pub fn create_projection_viewmatrix_euler(
    camera_position: &glm::Vec3,
    camera_euler_angles: &glm::Vec3,
    aspect_ratio: Option<f32>,
    field_of_view_y_degrees: Option<f32>,
    z_near: Option<f32>,
    z_far: Option<f32>
) -> glm::Mat4x4 {
    /*
        float pitch = -cameraEulerAngles.x * glm::pi<float>() / 180.0f;
        float yaw = cameraEulerAngles.y * glm::pi<float>() / 180.0f;
        float roll = cameraEulerAngles.z * glm::pi<float>() / 180.0f;
        glm::quat cameraRotation = glm::toQuat(glm::rotate(glm::rotate(glm::mat4(1.0f), yaw, UP_DIRECTION_UNIT), pitch, RIGHT_DIRECTION_UNIT));
    */
    let pitch = -camera_euler_angles.x * glm::pi::<f32>() / 180.;
    let yaw = camera_euler_angles.y * glm::pi::<f32>() / 180.;
    info!("pitch: {} yaw: {}", pitch, yaw);
    //let roll = camera_euler_angles.z * glm::pi() / 180.; // unused for now

    let camera_rotation = glm::to_quat(
        &glm::rotate(
            &glm::rotate(
                &glm::mat4(1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0,1.0),
                yaw,
                &*UP_DIRECTION_UNIT
            ),
            pitch,
            &*RIGHT_DIRECTION_UNIT
        )
    );
    create_projection_viewmatrix_quat(camera_position, &camera_rotation, aspect_ratio, field_of_view_y_degrees, z_near, z_far)
}

pub fn create_projection_viewmatrix_quat(
    camera_position: &glm::Vec3,
    camera_rotation: &glm::Quat,
    aspect_ratio: Option<f32>,
    field_of_view_y_degrees: Option<f32>,
    z_near: Option<f32>,
    z_far: Option<f32>
) -> glm::Mat4x4 {
/*
    // c++ code using the glm library
        float fieldOfViewYRadians = fieldOfViewYDegrees * glm::pi<float>() / 180.0f;
        glm::vec3 cameraUpDirectionUnit = glm::normalize(glm::vec3(cameraRotation * glm::vec4(UP_DIRECTION_UNIT, 1.0f))); /* affine transform, no need for homogeneous division */
        glm::mat4 projectionMatrix = glm::perspective(fieldOfViewYRadians, aspectRatio, zNear, zFar);
        glm::vec3 cameraTarget = cameraPosition + glm::vec3(cameraRotation * glm::vec4(FORWARD_DIRECTION_UNIT, 1.0f)); /* affine transform, no need for homogeneous division */
        glm::mat4 viewMatrix = glm::lookAt(cameraPosition, cameraTarget, cameraUpDirectionUnit);
        return projectionMatrix * viewMatrix;
*/
    let field_of_view_radians = field_of_view_y_degrees.unwrap_or(75.) * glm::pi::<f32>() / 180.;
    let camera_up_dir_unit = glm::normalize(
        &(camera_rotation * glm::quat(UP_DIRECTION_UNIT.x, UP_DIRECTION_UNIT.y, UP_DIRECTION_UNIT.z, 1.0)).as_vector().xyz()
    );
    let projection_matrix = glm::perspective(
        aspect_ratio.unwrap_or(16./9.),
        field_of_view_radians,
        z_near.unwrap_or(1.0),
        z_far.unwrap_or(4000.)
    );
    let camera_target = camera_position + (camera_rotation * glm::quat(FORWARD_DIRECTION_UNIT.x,FORWARD_DIRECTION_UNIT.y,FORWARD_DIRECTION_UNIT.z, 1.0)).as_vector().xyz();
    let view_matrix = glm::look_at_lh(camera_position, &camera_target, &camera_up_dir_unit);
    return projection_matrix * view_matrix;
}

pub fn transform_world_point_into_screen_space(world_point: &glm::Vec3, projection_view_matrix: &glm::Mat4x4, screen_width: Option<f32>, screen_height: Option<f32>) -> Option<glm::Vec2> {
    /*
        glm::vec4 screenPointHomogenousSpace = projectionViewMatrix * glm::vec4(worldPoint, 1.0f);
        glm::vec3 screenPoint = glm::vec3(screenPointHomogenousSpace.x, screenPointHomogenousSpace.y, screenPointHomogenousSpace.z) / screenPointHomogenousSpace.w;
        if ((screenPoint.x < -1.0f) || (screenPoint.x > 1.0f) || (screenPoint.y < -1.0f) || (screenPoint.y > 1.0f) || (screenPoint.z < -1.0f) || (screenPoint.z > 1.0f))
            return glm::vec2(std::nanf(""), std::nanf(""));
        return glm::vec2(screenWidth * (screenPoint.x + 1.0f) / 2.0f,
            screenHeight * (1.0f - (screenPoint.y + 1.0f) / 2.0f));
    */
    let screenpoint_homogenous_space = projection_view_matrix * glm::vec4(world_point.x, world_point.y, world_point.z, 1.0);
    let screen_point = screenpoint_homogenous_space.xyz() / screenpoint_homogenous_space.w;
    if screen_point.x < -1. || screen_point.x > 1. || screen_point.y < -1. || screen_point.y > 1. {
        None
    } else {
        Some(glm::vec2(
            screen_width.unwrap_or(1920.) * (screen_point.x + 1.0) / 2.0,
            screen_height.unwrap_or(1080.) * (1.0 - (screen_point.y + 1.0) / 2.0)
        ))
    }
}

pub fn is_world_point_visible_on_screen(world_point: &glm::Vec3, projection_view_matrix: &glm::Mat4x4) -> bool {
    return transform_world_point_into_screen_space(world_point, projection_view_matrix, Some(1.0), Some(1.0)).is_some()
}

/*
 // c++ code using the glm library
    glm::vec2 transformWorldPointIntoScreenSpace(const glm::vec3& worldPoint, const glm::mat4& projectionViewMatrix, float screenWidth, float screenHeight) {
        glm::vec4 screenPointHomogenousSpace = projectionViewMatrix * glm::vec4(worldPoint, 1.0f);
        glm::vec3 screenPoint = glm::vec3(screenPointHomogenousSpace.x, screenPointHomogenousSpace.y, screenPointHomogenousSpace.z) / screenPointHomogenousSpace.w;
        if ((screenPoint.x < -1.0f) || (screenPoint.x > 1.0f) || (screenPoint.y < -1.0f) || (screenPoint.y > 1.0f) || (screenPoint.z < -1.0f) || (screenPoint.z > 1.0f))
            return glm::vec2(std::nanf(""), std::nanf(""));
        return glm::vec2(screenWidth * (screenPoint.x + 1.0f) / 2.0f, screenHeight * (1.0f - (screenPoint.y + 1.0f) / 2.0f));
    }

    bool isWorldPointVisibleOnScreen(const glm::vec3& worldPoint, const glm::mat4& projectionViewMatrix) {
        return !glm::any(glm::isnan(transformWorldPointIntoScreenSpace(worldPoint, projectionViewMatrix, 1.0f, 1.0f)));
    }
*/

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

pub fn world_2_screen(world_pos: &glm::Vec3, view_matrix: &[[f32;4];4], screen_width: Option<f32>, screen_height: Option<f32>) -> Option<glm::Vec3> {
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
        Some(glm::vec3(
            (res_x * 0.5) + 0.5 * _x * res_x + 0.5,
            (res_y * 0.5) - 0.5 * _y * res_y + 0.5,
            inverse_w
        ))
    }
}

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
    d*(glm::pi::<f32>()/180.)
}