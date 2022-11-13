// this render thread takes in data such as player positions via a message sender and then does some gpu magic

mod texture;
mod camera;
mod instance;

use crate::{gamedata::minimap_info::{MapInfo, self}, datatypes::{tmp_vec3, tmp_vec2}, utils, render::instance::InstanceType};

use self::{
    camera::{Camera, CameraUniform},
    instance::{Instance, InstanceRaw},
};

use cgmath::{Rotation3, MetricSpace};
// gpu library
use wgpu::{include_wgsl, CompositeAlphaMode,util::DeviceExt, BindGroupLayout};
// fonts rendering library
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
// window creation
use winit::event_loop::EventLoopBuilder;
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::Fullscreen;
// other utils
use memflow::prelude::{Pod, PodMethods};
use std::{sync::mpsc, num::NonZeroU32};
use std::thread;

const MAX_INSTANCE_BUFFER_SIZE: u64 = (std::mem::size_of::<InstanceRaw>()*32) as u64;

#[derive(Default)]
pub struct PlayerLoc {
    pub world_pos: tmp_vec3,
    pub team: i32,
    pub name: String,
}

#[derive(Default)]
pub struct FrameData {
    pub connected: bool,
    pub local_position: PlayerLoc,
    pub locations: Vec<PlayerLoc>,
}

#[derive(Default)]
/// data update message for when there is a new map
pub struct MapData {
    pub map_name: Option<String>,
    pub map_details: Option<MapInfo>,
}

/// Text to be rendered on the map. Made up of a string and a vector location
struct MapText {
    text: String,
    loc: cgmath::Vector3<f32>,
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
// const PENTAGON_VERTICES: &[BufferVertex] = &[
//     BufferVertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
//     BufferVertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
//     BufferVertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
//     BufferVertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
//     BufferVertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
// ];

// const PENTAGON_INDICES: &[u16] = &[
//     0, 1, 4,
//     1, 2, 4,
//     2, 3, 4,
// ];

/// a plane mesh (2d square) along the x and y coordinates. From -1 to +1 (2 units size)
const PLANE_VERTICES: &[BufferVertex] = &[
    BufferVertex { position: [-1., -1., 0.0], tex_coords: [0., 1.], }, // A
    BufferVertex { position: [1., -1., 0.0], tex_coords: [1., 1.], }, // B
    BufferVertex { position: [1., 1., 0.0], tex_coords: [1., 0.], }, // C
    BufferVertex { position: [-1., 1., 0.0], tex_coords: [0., 0.], }, // D
];
/// the indices for our plane mesh
const PLANE_INDICES: &[u16] = &[
    0,1,2,
    2,3,0
];

/// the center origin for us to place our map mesh at. This is equal to half our map scale value (10)
const MAP_CENTER: (f32,f32) = (5.,-5.0);

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
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: wgpu::Features::TEXTURE_BINDING_ARRAY,
                        limits: Default::default(),
                    },
                    None
                )
                .await
                .expect("Request device")
        });

        let window_size =  window.inner_size();

        //
        // init camera
        //
        let mut camera = Camera {
            // position the camera 1 unit up and 50 units back
            eye: (MAP_CENTER.0,MAP_CENTER.1,15.0).into(),
            // have the camera look at the origin
            target: (0.0,0.0,0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: window_size.width as f32 / window_size.height as f32,
            fovy: 30.0,
            znear: 0.1,
            zfar: 2000.,
        };

        let mut player_minimap_location: tmp_vec3 = tmp_vec3::default();

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
                contents: PLANE_VERTICES.as_bytes(),
                usage: wgpu::BufferUsages::VERTEX,
            });
        //let num_verts = PLANE_VERTICES.len() as u32; // un used atm since we are using the whole buffer for this one mesh
        // create index vector
        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: PLANE_INDICES.as_bytes(),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = PLANE_INDICES.len() as u32;

        // instance buffer
        //let test_data = Instance::make_test_data(f64::sin(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())).iter().map(Instance::to_raw).collect::<Vec<_>>();
        //let mut instance_data = vec![];
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: MAX_INSTANCE_BUFFER_SIZE,
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
        let no_map_bytes = include_bytes!("../../assets/maps/no_map.png");
        let t_diffuse_bytes = include_bytes!("../../assets/textures/t.png");
        let ct_diffuse_bytes = include_bytes!("../../assets/textures/ct.png");
        let local_diffuse_bytes = include_bytes!("../../assets/textures/target.png");
        
        let mut map_diffuse_texture = texture::Texture::from_bytes(&device, &queue, no_map_bytes, "map.png").unwrap();
        let t_diffuse_texture = texture::Texture::from_bytes(&device, &queue, t_diffuse_bytes, "t.png").unwrap();
        let ct_diffuse_texture = texture::Texture::from_bytes(&device, &queue, ct_diffuse_bytes, "ct.png").unwrap();
        let local_diffuse_texture = texture::Texture::from_bytes(&device, &queue, local_diffuse_bytes, "local.png").unwrap();

        // create bind group
        // this describes a set of resources and how they may be accessed by a shader.

        // TODO: REPLACE THESE MANY BINDINGS WITH ONE TEXTURE ARRAY BINDING

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
                        count: NonZeroU32::new(4),
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        //this should match the filterable field of the corresponding texture entry above
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: NonZeroU32::new(4),
                    },
                ],
            });

        // bind group for the t texture
        let mut texture_bind_group = update_texture_bind_group(
            &device,
            &texture_bind_group_layout,
            &map_diffuse_texture,
            &local_diffuse_texture,
            &t_diffuse_texture,
            &ct_diffuse_texture
        );
        
        // bind group for the t texture
        // let t_diffuse_bind_group = device.create_bind_group(
        //     &wgpu::BindGroupDescriptor {
        //         label: Some("t_diffuse_bind_group"),
        //         layout: &texture_bind_group_layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: wgpu::BindingResource::TextureView(&t_diffuse_texture.view),
        //             },
        //             wgpu::BindGroupEntry {
        //                 binding: 1,
        //                 resource: wgpu::BindingResource::Sampler(&t_diffuse_texture.sampler),
        //             },
        //         ],
        //     }
        // );

        // let ct_diffuse_bind_group = device.create_bind_group(
        //     &wgpu::BindGroupDescriptor {
        //         label: Some("ct_diffuse_bind_group"),
        //         layout: &texture_bind_group_layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: wgpu::BindingResource::TextureView(&ct_diffuse_texture.view),
        //             },
        //             wgpu::BindGroupEntry {
        //                 binding: 1,
        //                 resource: wgpu::BindingResource::Sampler(&ct_diffuse_texture.sampler),
        //             },
        //         ],
        //     }
        // );

        // let local_diffuse_bind_group = device.create_bind_group(
        //     &wgpu::BindGroupDescriptor {
        //         label: Some("local_diffuse_bind_group"),
        //         layout: &texture_bind_group_layout,
        //         entries: &[
        //             wgpu::BindGroupEntry {
        //                 binding: 0,
        //                 resource: wgpu::BindingResource::TextureView(&local_diffuse_texture.view),
        //             },
        //             wgpu::BindGroupEntry {
        //                 binding: 1,
        //                 resource: wgpu::BindingResource::Sampler(&local_diffuse_texture.sampler),
        //             },
        //         ],
        //     }
        // );

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

        // depth stencil state for stencil testing
        // let stencil_state = wgpu::StencilFaceState {
        //     compare: wgpu::CompareFunction::Always,
        //     fail_op: wgpu::StencilOperation::Keep,
        //     depth_fail_op: wgpu::StencilOperation::Keep,
        //     pass_op: wgpu::StencilOperation::IncrementClamp,
        // };

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
            label: Some("2D Render Pipeline"),
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
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // TODO: change this to point list later
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,//Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default()/*{
                    front: stencil_state,
                    back: stencil_state,
                    read_mask: 0xff,
                    write_mask: 0xff,
                }*/,
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

        // stores a list of name labels to be rendered and their minimap locations
        let mut name_list: Vec<MapText> = vec![];

        //
        // Start the render and event loop
        // 

        // render loop
        window.request_redraw();

        event_loop.run(move |event, _, control_flow| {
            // this is to make sure that resources are cleaned up properly.
            // Since event loop run never returns we need it to take ownership of resources
            let _ = (&instance,&shader,&render_pipeline_layout, &vertex_buffer, &camera_buffer, &index_buffer, &instance_buffer);

            let angle = 25.;//f64::sin(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())*30.;

            // first update the frame data if it was received
            // while loop through all the receive data to get to the latest in case a buildup happens
            let mut new_frame = None;
            while let Ok(dat) = rx.try_recv() {
                new_frame = Some(dat);
            }
            if let Some(frame) = new_frame {
                framedata = frame;

                let mut new_text_spots = Vec::with_capacity(framedata.locations.len());
                let instance_data = {
                    use cgmath::{Vector3,Quaternion};
                    let mut new_instances = Vec::with_capacity(framedata.locations.len());

                    // map instance
                    new_instances.push(Instance{
                        position: (MAP_CENTER.0,MAP_CENTER.1,0.0).into(),
                        rotation: Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(0.)),
                        // scaled by 5 because oops i made a plane 2x units so scale of 5 equals a plane of 10 units size
                        // might change this later but for now the plane remains two units in size (-1 to +1)
                        scale: (5.,5.,1.).into(),
                        instance_type: InstanceType::MapTexture,
                    });

                    // local player

                    new_instances.push(Instance{
                        position: (player_minimap_location.x,player_minimap_location.y,0.3).into(),
                        rotation: Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(0.)),
                        // scale the player by half to make it one unit in size instead of two
                        scale: (0.5,0.5,1.).into(),
                        instance_type: InstanceType::LocalPlayer,
                    });

                    for (_, data) in framedata.locations.iter().enumerate() {
                        let pos = 
                        {
                            let map_detail = map_data.map_details.unwrap_or(MapInfo {
                                pos_x: -2796.0,
                                pos_y: 3328.0,
                                scale: 5.0,
                            });
                            let pos = utils::math::radar_scale(
                                data.world_pos.x,
                                data.world_pos.y,
                                map_detail.scale,
                                map_detail.pos_x,
                                map_detail.pos_y,
                                Some((10.,10.))
                            );
                            
                            // accounts for them being slightly out of position visually when not flat / origin is center of sprite and not the feet
                            let y_offset = 0.14;
                            Vector3 { x: pos.0, y: pos.1 + y_offset, z: 0.5 }
                        };
                        
                        // push a new instance to be rendered
                        new_instances.push(Instance{
                            position: pos,
                            rotation: Quaternion::from_axis_angle(cgmath::Vector3::unit_x(), cgmath::Deg(angle as f32)),
                            // scale the enemies to be half a unit in size
                            scale: (0.25,0.25,1.).into(),
                            instance_type: data.team.into(),
                        });

                        // push a new text tag to be rendered
                        new_text_spots.push(MapText {
                            text: data.name.clone(),
                            loc: pos,
                        });
                    }
                    // z sort the data before render 
                    // the location we wanna check distance to
                    let sort_from: Vector3<f32> = (MAP_CENTER.0,-20.,1.0).into();
                    new_instances.sort_by(|a,b| {
                        a.position.distance2(sort_from).partial_cmp(&b.position.distance2(sort_from)).unwrap()
                    });
                    new_instances
                }.iter().map(Instance::to_raw).collect::<Vec<_>>();
                //let instance_data = Instance::make_test_data(f64::sin(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64())*90.).iter().map(Instance::to_raw).collect::<Vec<_>>();
                num_instances = instance_data.len() as u32;
                name_list = new_text_spots;

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

                // load new map texture and update the bind group
                if let Some(map_name) = &map_data.map_name {
                    map_diffuse_texture = texture::Texture::from_image(
                        &device,
                        &queue,
                        &minimap_info::load_map_image(map_name.clone()).unwrap_or(image::load_from_memory(no_map_bytes).unwrap()),
                        Some("map.png")
                    ).expect("loading the map texture from image bytes");

                    // update the texture array bind group
                    texture_bind_group = update_texture_bind_group(
                        &device,
                        &texture_bind_group_layout,
                        &map_diffuse_texture,
                        &local_diffuse_texture,
                        &t_diffuse_texture,
                        &ct_diffuse_texture
                    );
                }
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

                    //
                    // update camera uniform
                    //
                    let wpos = framedata.local_position.world_pos;
                    player_minimap_location =
                        {
                            let map_detail = map_data.map_details.unwrap_or(MapInfo {
                                pos_x: -2796.0,
                                pos_y: 3328.0,
                                scale: 5.0,
                            });
                            let pos = utils::math::radar_scale(
                                wpos.x,
                                wpos.y,
                                map_detail.scale,
                                map_detail.pos_x,
                                map_detail.pos_y,
                                Some((10.,10.))
                            );
                            tmp_vec3 { x: pos.0, y: pos.1, z: 0.5 }
                        };

                    // set camera target to half way between map center and the current player location
                    camera.target = {
                        let diff = player_minimap_location.xy() - tmp_vec2::from(MAP_CENTER);
                        let halfway = player_minimap_location - (diff/2.);
                        (halfway.x,halfway.y,0.0).into()
                    };
                    

                    camera_uniform.update_view_proj(&camera);
                    let camera_staging_buffer = device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Camera Buffer"),
                            contents: camera_uniform.as_bytes(),
                            usage: wgpu::BufferUsages::COPY_SRC,
                        }
                    );
                    encoder.copy_buffer_to_buffer(&camera_staging_buffer, 0, &camera_buffer, 0, camera_staging_buffer.size());

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
                                                r: 0.03,
                                                g: 0.03,
                                                b: 0.05,
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
                                    stencil_ops: None/*Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(0),
                                        store: true,
                                    })*/,
                                }),
                            });

                        render_pass.set_pipeline(&render_pipeline);
                        // add the texture bind group
                        render_pass.set_bind_group(0, &texture_bind_group, &[]);
                        // render_pass.set_bind_group(1, &t_diffuse_bind_group, &[]);
                        // render_pass.set_bind_group(2, &ct_diffuse_bind_group, &[]);
                        // render_pass.set_bind_group(3, &local_diffuse_bind_group, &[]);
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
                        screen_position: (30.0, 30.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(
                            format!("status: {} map {} cam center: {} {}", if framedata.connected {"connected"} else {"waiting"}, map_data.map_name.clone().unwrap_or("none".to_string()), camera.target.x,camera.target.y).as_str(),
                        )
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(24.0)],
                        ..Section::default()
                    });

                    // let player_screen = camera.build_view_projection_matrix() * cgmath::Vector4::new(4.69,-4.69,0.0, 1.);
                    // let inverse_w = 1. / player_screen.w;
                    // let (x,y) = (
                    //     (size.width as f32 * 0.5) + 0.5 * (player_screen.x*inverse_w) * size.width as f32 + 0.5,
                    //     (size.height as f32 * 0.5) - 0.5 * (player_screen.y*inverse_w) * size.height as f32 + 0.5
                    // );
                    let projection = camera.build_view_projection_matrix();
                    let (x,y) = project(
                        projection,
                        cgmath::Vector3::new(player_minimap_location.x,player_minimap_location.y+0.4,1.0),
                        size.width as f32,
                        size.height as f32
                    );
                    glyph_brush.queue(Section {
                        screen_position: (x, y),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(
                            "you",
                        )
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(12.0)],
                        ..Section::default()
                    });

                    for (_,data) in name_list.iter().enumerate() {
                        let projection = camera.build_view_projection_matrix();
                        let (x,y) = project(
                            projection,
                            data.loc + cgmath::Vector3::new(-0.3,0.4,0.2),
                            size.width as f32,
                            size.height as f32
                        );
                        glyph_brush.queue(Section {
                            screen_position: (x, y),
                            bounds: (size.width as f32, size.height as f32),
                            text: vec![Text::new(
                                data.text.as_str(),
                            )
                            .with_color([1.0, 1.0, 1.0, 1.0])
                            .with_scale(12.0)],
                            ..Section::default()
                        });
                    }

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

fn update_texture_bind_group(
    device: &wgpu::Device,
    texture_bind_group_layout: &BindGroupLayout,
    map_diffuse_texture: &texture::Texture,
    local_diffuse_texture: &texture::Texture,
    t_diffuse_texture: &texture::Texture,
    ct_diffuse_texture: &texture::Texture,
) -> wgpu::BindGroup {
    device.create_bind_group(
        &wgpu::BindGroupDescriptor {
            label: Some("map_diffuse_bind_group"),
            layout: texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&[
                            &map_diffuse_texture.view,
                            &local_diffuse_texture.view,
                            &t_diffuse_texture.view,
                            &ct_diffuse_texture.view,
                        ]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::SamplerArray(&[
                            &map_diffuse_texture.sampler,
                            &local_diffuse_texture.sampler,
                            &t_diffuse_texture.sampler,
                            &ct_diffuse_texture.sampler,
                        ]),
                },
            ],
        }
    )
}

fn project(projection_matrix: cgmath::Matrix4<f32>, map_pos: cgmath::Vector3<f32>, out_width: f32, out_height: f32) -> (f32,f32) {
    let player_screen = projection_matrix * cgmath::Vector4::new(map_pos.x,map_pos.y,map_pos.z, 1.);
    let inverse_w = 1. / player_screen.w;
    (
        (out_width * 0.5) + 0.5 * (player_screen.x*inverse_w) * out_width + 0.5,
        (out_height * 0.5) - 0.5 * (player_screen.y*inverse_w) * out_height + 0.5
    )
}