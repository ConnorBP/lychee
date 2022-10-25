// Some math stuff for viewmatricies and such

use lazy_static::lazy_static;

lazy_static! {
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
        z_near.unwrap_or(0.8),
        z_far.unwrap_or(4000.)
    );
    let camera_target = camera_position + (camera_rotation * glm::quat(FORWARD_DIRECTION_UNIT.x,FORWARD_DIRECTION_UNIT.y,FORWARD_DIRECTION_UNIT.z, 1.0)).as_vector().xyz();
    let view_matrix = glm::look_at(camera_position, &camera_target, &camera_up_dir_unit);
    return projection_matrix * view_matrix;
}

pub fn transform_world_point_into_screen_space(world_point: &glm::Vec3, projection_view_matrix: &glm::Mat4x4, screen_width: Option<f32>, screen_height: Option<f32>) -> Option<glm::Vec2> {
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