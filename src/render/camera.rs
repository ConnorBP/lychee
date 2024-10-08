

/*

The coordinate system in Wgpu is based on DirectX, and Metal's coordinate systems.
That means that in normalized device coordinates (opens new window)
the x axis and y axis are in the range of -1.0 to +1.0, and the z axis is 0.0 to +1.0.
The cgmath crate (as well as most game math crates) is built for OpenGL's coordinate system.
This matrix will scale and translate our scene from OpenGL's coordinate system to WGPU's. 
We'll define it as follows:

*/

use cgmath::{Quaternion, Vector3, InnerSpace, Rad};
use memflow::prelude::Pod;

// basically a magic matrix
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: Option<cgmath::Point3<f32>>,
    pub rotation: Quaternion<f32>,
    pub offset: Vector3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub pitch: cgmath::Rad<f32>,
    pub yaw: cgmath::Rad<f32>,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = if let Some(target) = self.target {
            cgmath::Matrix4::look_at_rh(self.eye, target, self.up)
        } else {
            let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
            let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();
            cgmath::Matrix4::look_to_lh(
                self.eye,
                Vector3 { x: -cos_pitch * cos_yaw, y: sin_pitch, z: cos_pitch * sin_yaw }.normalize(),
                // Vector3 { x: cos_pitch * cos_yaw, y: cos_pitch * sin_yaw, z: sin_pitch }.normalize(),
                self.up,
            )
        };
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        let rot = cgmath::Matrix4::from(self.rotation);
        let offset = cgmath::Matrix4::from_translation(self.offset);
        let mut final_mat = OPENGL_TO_WGPU_MATRIX * proj * offset * rot * view;
        // final_mat.w.x = 2.0 * (720.0 / 1920.0); // width if not rotated
        // final_mat.w.x -= 2.5;
        // final_mat.w.y = 2.0 * 720.0 / 1920.0; // height if not rotated
        // final_mat.w.y += 2.5;
        //final_mat.w.z = 2.0 * 720.0 / 1920.0;
        return final_mat;
    }
    pub fn update_window_size(&mut self, x:f32,y:f32)
    {
        self.aspect = x/y;
    }
}

#[repr(C)]
#[derive(Debug,Copy,Clone,Pod)]
pub struct CameraUniform {
    pub view_proj: [[f32;4];4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
