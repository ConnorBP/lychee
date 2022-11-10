// this render thread takes in data such as player positions via a message sender and then does some gpu magic

mod texture;
mod camera;
mod instance;

use crate::{gamedata::minimap_info::MapInfo, datatypes::tmp_vec3, utils};

use self::{
    camera::{Camera, CameraUniform},
    instance::{Instance, InstanceRaw},
};

use cgmath::Rotation3;
use image::GenericImageView;
// gpu library
use wgpu::{include_wgsl, CompositeAlphaMode,util::DeviceExt};
// fonts rendering library
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
// window creation
use winit::event_loop::EventLoopBuilder;
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::Fullscreen;
// other utils
use memflow::prelude::Pod;
use std::{sync::mpsc, time::SystemTime};
use std::thread;

const MAX_INSTANCE_BUFFER_SIZE: u64 = (std::mem::size_of::<InstanceRaw>()*32) as u64;

pub struct PlayerLoc {
    pub world_pos: tmp_vec3,
    pub head_pos: Option<glm::Vec3>,
    pub feet_pos: Option<glm::Vec3>,
    pub team: i32,
}

#[derive(Default)]
pub struct FrameData {
    pub connected: bool,
    pub locations: Vec<PlayerLoc>,
}

#[derive(Default)]
/// data update message for when there is a new map
pub struct MapData {
    pub map_name: Option<String>,
    pub map_details: Option<MapInfo>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod)]
struct BufferVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl BufferVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BufferVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// temp const test vertice array
const VERTICES: &[BufferVertex] = &[
    BufferVertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
    BufferVertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
    BufferVertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
    BufferVertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
    BufferVertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

pub fn start_window_render(
) -> std::result::Result<(mpsc::Sender<FrameData>,mpsc::Sender<MapData>), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<FrameData>();
    let (map_tx, map_rx) = mpsc::channel::<MapData>();

    thread::spawn(|| {
        // our frame data to be rendered (a list of player screen positions)
        let mut framedata = FrameData::default();
        // info about the currently played map
        let mut map_data = MapData::default();

        let event_loop = EventLoopBuilder::new()
            .with_any_thread(true)
            .with_dpi_aware(false)
            .build();

        let window = winit::window::WindowBuilder::new()
            .with_title("lyche radar")
            .with_resizable(false)
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let (device, queue) = futures::executor::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .expect("Request adapter");
            adapter
                .request_device(&wgpu::DeviceDescriptor::default(), None)
                .await
                .expect("Request device")
        });

        let window_size =  window.inner_size();

        //
        // init camera
        //
        let mut camera = Camera {
            // position the camera 1 unit up and 50 units back
            eye: (0.0,1.0,10.0).into(),
            // have the camera look at the origin
            target: (0.0,0.0,0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: window_size.width as f32 / window_size.height as f32,
            fovy: 75.0,
            znear: 0.1,
            zfar: 2000.,
        };

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: camera_uniform.as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        // our vertex buffer to send to the gpu each frame
        let vertex_buffer: wgpu::Buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: VERTICES.as_bytes(),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let num_verts = VERTICES.len() as u32;
        // create index vector
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: INDICES.as_bytes(),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;

        // instance buffer
        //let test_data = Instance::make_test_data(f64::sin(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())).iter().map(Instance::to_raw).collect::<Vec<_>>();
        //let mut instance_data = vec![];
        let mut instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: 6400,//MAX_INSTANCE_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // let instance_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Instance Buffer"),
        //         contents: test_data.as_bytes(),
        //         usage: wgpu::BufferUsages::VERTEX
        //     }
        // );
        let mut staging_instance_buffer = None;
        let mut num_instances = 0;

        // load the shader
        let shader = device.create_shader_module(include_wgsl!("../../assets/shaders/shader.wgsl"));

        // create staging belt
        let mut staging_belt = wgpu::util::StagingBelt::new(1024);

        // prepare swap chain
        let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut size = window.inner_size();

        //
        // Texture Init
        //

        // depth texture
        let mut depth_texture = texture::Texture::create_depth_texture(&device, &size, "depth_texture");

        // prepare the textures
        let t_diffuse_bytes = include_bytes!("../../assets/textures/t.png");
        let ct_diffuse_bytes = include_bytes!("../../assets/textures/ct.png");

        let t_diffuse_texture = texture::Texture::from_bytes(&device, &queue, t_diffuse_bytes, "t.png").unwrap();

        // create bind group
        // this describes a set of resources and how they may be accessed by a shader.

        // bind group layout (shared for all textures)
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {filterable: true},
                            view_dimension:wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        //this should match the filterable field of the corresponding texture entry above
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        
        // bind group for the t texture
        let t_diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("t_diffuse_bind_group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&t_diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&t_diffuse_texture.sampler),
                    },
                ],
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
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
        });

        //
        // Render Pipeline
        //

        // create the render pipeline layout

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout,&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        // make render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shader Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    BufferVertex::desc(), InstanceRaw::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: render_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // TODO: change this to point list later
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: render_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                alpha_mode: CompositeAlphaMode::Auto,
            },
        );

        //
        // Font Init
        //

        // prepare the glyph_brush
        let white_rabbit =
            ab_glyph::FontArc::try_from_slice(include_bytes!("../../assets/fonts/whitrabt.ttf"))
                .expect("could not load font");

        let mut glyph_brush =
            GlyphBrushBuilder::using_font(white_rabbit).build(&device, render_format);

        //
        // Start the render and event loop
        // 

        // render loop
        window.request_redraw();

        event_loop.run(move |event, _, control_flow| {
            // this is to make sure that resources are cleaned up properly.
            // Since event loop run never returns we need it to take ownership of resources
            let _ = (&instance,&shader,&render_pipeline_layout);

            // first update the frame data if it was received
            if let Ok(frame) = rx.try_recv() {
                framedata = frame;

                // let instance_data = {
                //     use cgmath::{Vector3,Quaternion};
                //     let mut new_instances = Vec::with_capacity(framedata.locations.len());
                //     for (i, data) in framedata.locations.iter().enumerate() {
                //         new_instances.push(Instance{
                //             position: Vector3 { x: data.world_pos .x, y: data.world_pos .y, z: data.world_pos .z },
                //             rotation: Quaternion::from_axis_angle(cgmath::Vector3::unit_y(), cgmath::Deg(0.0)),
                //         });
                //     }
                //     new_instances
                // }.iter().map(Instance::to_raw).collect::<Vec<_>>();
                let instance_data = Instance::make_test_data(f64::sin(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())*90.).iter().map(Instance::to_raw).collect::<Vec<_>>();
                num_instances = instance_data.len() as u32;

                // now update textures and bindings and such

                staging_instance_buffer = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: instance_data.as_bytes(),
                        usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                    }
                ));

                // request a redraw if we got new info
                window.request_redraw();
            }

            // if the map info has changed update the required textures acordingly
            if let Ok(new_map_data) = map_rx.try_recv() {
                map_data = new_map_data;
            }

            match event {
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CloseRequested,
                    ..
                } => *control_flow = winit::event_loop::ControlFlow::Exit,
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(new_size),
                    ..
                } => {
                    size = new_size;
                    surface.configure(
                        &device,
                        &wgpu::SurfaceConfiguration {
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            format: render_format,
                            width: size.width,
                            height: size.height,
                            present_mode: wgpu::PresentMode::AutoNoVsync,
                            alpha_mode: CompositeAlphaMode::Auto,
                        },
                    );
                    camera.update_window_size(size.width as f32, size.height as f32);
                    // re create the depth texture with the new window size
                    depth_texture = texture::Texture::create_depth_texture(&device, &size, "depth_texture");
                }
                winit::event::Event::RedrawRequested { .. } => {
                    // Get a command encoder for the current frame
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Redraw"),
                        });
                    // if instance staging buffer has data then copy it into the instance buffer
                    if let Some(stage) = &staging_instance_buffer {
                        encoder.copy_buffer_to_buffer(stage, 0, &instance_buffer, 0, utils::math::round_up(stage.size(), wgpu::COPY_BUFFER_ALIGNMENT));
                        staging_instance_buffer = None;
                    }

                    // get the next frame
                    let frame = surface.get_current_texture().expect("get next frame");
                    let view = &frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    // clear frame
                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Render pass"),
                                color_attachments: &[Some(
                                    // this is what @location(0) in the fragment shader targets
                                    wgpu::RenderPassColorAttachment {
                                        view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                                r: 0.4,
                                                g: 0.4,
                                                b: 0.2,
                                                a: 1.0,
                                            }),
                                            store: true,
                                        },
                                    },
                                )],
                                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                    view: &depth_texture.view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: true,
                                    }),
                                    stencil_ops: None,
                                }),
                            });

                        render_pass.set_pipeline(&render_pipeline);
                        // add the texture bind group
                        render_pass.set_bind_group(0, &t_diffuse_bind_group, &[]);
                        // add the camera bind group
                        render_pass.set_bind_group(1, &camera_bind_group, &[]);
                        // set the vertex buffer
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        // set the second vertex buffer as the instance buffer
                        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                        // set the index buffer
                        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        // finally call draw but with indexed
                        render_pass.draw_indexed(0..num_indices, 0, 0..num_instances);
                        //draw one instance of 3 vertices
                        // render_pass.draw(0..num_verts, 0..1);
                    }

                    glyph_brush.queue(Section {
                        screen_position: (30.0, 90.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(
                            format!("connected: {} map {:?}", framedata.connected, map_data.map_name.clone().unwrap_or("none".to_string())).as_str(),
                        )
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(40.0)],
                        ..Section::default()
                    });

                    // draw the text
                    glyph_brush
                        .draw_queued(
                            &device,
                            &mut staging_belt,
                            &mut encoder,
                            view,
                            size.width,
                            size.height,
                        )
                        .expect("Draw queued");
                    // submit the work
                    staging_belt.finish();
                    queue.submit(Some(encoder.finish()));
                    frame.present();
                    // recall unused staging buffers
                    staging_belt.recall();
                }
                _ => {
                    // for any other control flows do a wait
                    //*control_flow = winit::event_loop::ControlFlow::Wait;
                }
            }
        }) // end of event loop
    }); // thread
    Ok((tx,map_tx)) // return the sender after we create the thread
}
