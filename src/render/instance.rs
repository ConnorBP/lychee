use cgmath::prelude::*;
use memflow::prelude::Pod;

pub enum InstanceType {
    MapTexture,
    LocalPlayer,
    TPlayer,
    CTPlayer,
}

impl From<i32> for InstanceType {
    fn from(v: i32) -> Self {
        match v {
            x if x == InstanceType::LocalPlayer as i32 => InstanceType::LocalPlayer,
            x if x == InstanceType::TPlayer as i32 => InstanceType::TPlayer,
            x if x == InstanceType::CTPlayer as i32 => InstanceType::CTPlayer,
            _ => InstanceType::MapTexture
        }
    }
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: cgmath::Vector3<f32>,
    pub instance_type: InstanceType,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw { 
            model: (
                cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation)
                * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
            ).into(),
            details: [self.instance_type as i32,0,0,0]
        }
    }
    #[allow(dead_code)]
    pub fn make_test_data<'a>(angle: f64) -> Vec<Instance> {
        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0., NUM_INSTANCES_PER_ROW as f32 * 0.5);
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(angle as f32))
                };

                Instance {
                    position,
                    rotation,
                    scale: (1.,1.,1.).into(),
                    instance_type: InstanceType::TPlayer,
                }
            })
        }).collect::<Vec<Instance>>();
        instances
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod)]
pub struct InstanceRaw {
    model: [[f32;4];4],
    details: [i32;4],
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // a mat4 takes up 4 vertex slots. We will re assemble it in the shader
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Sint32x4,
                },
            ],
        }
    }
}