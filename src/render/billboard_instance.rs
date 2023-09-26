use memflow::prelude::Pod;

pub struct BillboardInstance {
    pub position: cgmath::Vector3<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub color: cgmath::Vector4<f32>,
}

impl BillboardInstance {
    pub fn to_raw(&self) -> BillboardInstanceRaw {
        BillboardInstanceRaw { 
            center_pos: self.position.into(),
            size: self.scale.into(),
            color: self.color.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod)]
pub struct BillboardInstanceRaw {
    center_pos: [f32;3],
    size: [f32;2],
    color: [f32;4],
}

impl BillboardInstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BillboardInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // a mat4 takes up 4 vertex slots. We will re assemble it in the shader
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;5]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}