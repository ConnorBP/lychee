use std::f32::consts::PI;
use std::time::SystemTime;
use std::borrow::Cow;

use cgmath::{Rotation3, Rad};
use wgpu::{RenderPipeline, LoadOp, util::DeviceExt};
use memflow::dataview::PodMethods;

use super::PLANE_INDICES;

use super::camera::RotationUniform;
use super::{billboard_instance::{BillboardInstanceRaw, BillboardInstance}, camera::{CameraUniform, Camera}};

const MAX_BOXES : usize = 64;
const MAX_BOX_INSTANCE_BUFFER_SIZE: u64 = (std::mem::size_of::<BillboardInstanceRaw>()*MAX_BOXES) as u64;

// #[derive(Debug, Clone)]
// pub enum BoxPassError{
//     RenderError,
// }
// impl fmt::Display for BoxPassError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "error in box render pass")
//     }
// }
// pub type Result<T> = std::result::Result<T, BoxPassError>;

pub struct BoxPass {
    render_pipeline: RenderPipeline,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instance_buffer: wgpu::Buffer,
    pub instances: Vec<BillboardInstance>,
    // Camera stuff
    pub camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    // rotation uniform for the billboards cause i couldn't get anything else to work
    pub rotation: Rad<f32>,
    rotation_uniform: RotationUniform,
    rotation_buffer: wgpu::Buffer,

    // for delta
    start: SystemTime,
}

impl BoxPass {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        output_format: wgpu::TextureFormat,
        // msaa_samples: u32,
    ) -> Self {
         // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("box shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../../assets/shaders/box3dpoint.wgsl"))),
        });

        //
        // init camera
        //
        let camera = Camera {
            // position the camera 1 unit up and 50 units back
            eye: (0.0,10.0,15.0).into(),
            // have the camera look at the origin
            target: None,
            rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(0.)),
            up: cgmath::Vector3::unit_y(),
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 75.0,
            znear: 0.1,
            zfar: 2000.,
            offset: cgmath::Vector3::new(0.0, 0.0, 0.0),
            pitch: Rad(0.0),
            yaw: Rad(0.0),
        };

        
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("ESP Camera Buffer"),
                contents: camera_uniform.as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let rotation_uniform = RotationUniform::new();

        let rotation_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("ESP Rotation Buffer"),
                contents: rotation_uniform.as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        //
        // camera projection matrix bind group
        //


        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },

                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout_boxes"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group_boxes"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rotation_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Box Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    BillboardInstanceRaw::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format:None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,//Some(wgpu::Face::Back),
                unclipped_depth:false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative:false,
              },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            multiview: None,
        });

        // create index vector
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Box Index Buffer"),
                contents: PLANE_INDICES.as_bytes(),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = PLANE_INDICES.len() as u32;

        let instances = vec![
            BillboardInstance {
                position: cgmath::Vector3 { x: 0.5, y: 0.5, z: 0.0 },
                rotation: camera.rotation,
                scale: cgmath::Vector3 { x: 0.25, y: 1.0, z: 0.25 },
                color: (0.5,1.0,1.0,1.0).into(),
            },
            BillboardInstance {
                position: cgmath::Vector3 { x: 4.0, y: 0.5, z: 1.0 },
                rotation: camera.rotation,
                scale: cgmath::Vector3 { x: 1.0, y: 0.25, z: 0.25 },
                color: (1.0,0.5,0.5, 1.0).into(),
            },
        ];

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: MAX_BOX_INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            render_pipeline,
            // vertex_buffer,
            index_buffer,
            num_indices,
            instance_buffer,
            instances,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            rotation: Rad(0.0),
            rotation_uniform,
            rotation_buffer,

            start: SystemTime::now(),
        }
    }
    fn update_buffers(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // update camera uniform buffer
        let camera_staging_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Staging Buffer"),
                contents: self.camera_uniform.as_bytes(),
                usage: wgpu::BufferUsages::COPY_SRC,
            }
        );
        encoder.copy_buffer_to_buffer(&camera_staging_buffer, 0, &self.camera_buffer, 0, camera_staging_buffer.size());
        //update rotation buffer
        let rot_staging_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Rotation Staging Buffer"),
                contents: self.rotation_uniform.as_bytes(),
                usage: wgpu::BufferUsages::COPY_SRC,
            }
        );
        encoder.copy_buffer_to_buffer(&rot_staging_buffer, 0, &self.rotation_buffer, 0, rot_staging_buffer.size());
        // update instances buffer
        if !self.instances.is_empty() {
            let staging_instance_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Staging Buffer"),
                    contents: self.instances.iter().map(BillboardInstance::to_raw).collect::<Vec<_>>().as_bytes(),
                    usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                }
            );
            encoder.copy_buffer_to_buffer(
                &staging_instance_buffer,
                0,
                &self.instance_buffer,
                0,
                crate::utils::math::round_up(
                    staging_instance_buffer.size(),
                    wgpu::COPY_BUFFER_ALIGNMENT
                )
            );
        }
    }

    pub fn execute(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_attachment: &wgpu::TextureView,
        clear_color: Option<wgpu::Color>,
    ) {
        {

            // set testing camera pos
            
            // let time = SystemTime::now().duration_since(self.start).unwrap().as_secs_f32();
            // self.camera.eye = (f32::sin(time*0.2)*15.0,10.0,f32::sin(time*0.1-20.0)*15.0).into();
            self.camera_uniform.update_view_proj(&self.camera);
            self.rotation_uniform.update_rotation(self.rotation);

            self.update_buffers(device, encoder);
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_attachment,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: clear_color.map_or(LoadOp::Load, |c| LoadOp::Clear(c)),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.camera_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);

        }
    }
}