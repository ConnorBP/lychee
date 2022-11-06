// this render thread takes in data such as player positions via a message sender and then does some gpu magic

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
use std::sync::mpsc;
use std::thread;

pub struct PlayerLoc {
    pub head_pos: Option<glm::Vec3>,
    pub feet_pos: Option<glm::Vec3>,
    pub team: i32,
}

#[derive(Default)]
pub struct FrameData {
    pub connected: bool,
    pub locations: Vec<PlayerLoc>,
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
    BufferVertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.99240386], }, // A
    BufferVertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.56958647], }, // B
    BufferVertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.05060294], }, // C
    BufferVertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.1526709], }, // D
    BufferVertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.7347359], }, // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

pub fn start_window_render(
) -> std::result::Result<mpsc::Sender<FrameData>, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<FrameData>();

    thread::spawn(|| {
        // our frame data to be rendered (a list of player screen positions)
        let mut framedata = FrameData::default();

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

        // load the shader
        let shader = device.create_shader_module(include_wgsl!("../assets/shaders/shader.wgsl"));

        // create staging belt
        let mut staging_belt = wgpu::util::StagingBelt::new(1024);

        // prepare swap chain
        let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut size = window.inner_size();

        //
        // Texture Init
        //

        // prepare the textures
        let (t_rgba, t_dmimensions) = {
            let diffuse_bytes = include_bytes!("../assets/textures/t.png");
            let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
            (diffuse_image.to_rgba8(), diffuse_image.dimensions())
        };
        let (ct_rgba, ct_dmimensions) = {
            let diffuse_bytes = include_bytes!("../assets/textures/ct.png");
            let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
            (diffuse_image.to_rgba8(), diffuse_image.dimensions())
        };

        let t_texture_size = wgpu::Extent3d {
            width: t_dmimensions.0,
            height: t_dmimensions.1,
            depth_or_array_layers: 1,
        };

        let t_diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored in 3D, we represent our 2d tex by setting depth to 1
                size: t_texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // texture binding means we want to use this in shaders
                // copy_dst means we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("t_diffuse_texture"),

            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture{
                texture: &t_diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &t_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4*t_dmimensions.0),
                rows_per_image: std::num::NonZeroU32::new(t_dmimensions.1)
            },
            t_texture_size
        );

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let t_diffuse_texture_view = t_diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,// pixel art style filtering
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

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
                        resource: wgpu::BindingResource::TextureView(&t_diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    },
                ],
            }
        );

        //
        // Render Pipeline
        //

        // create the render pipeline layout

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
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
                    BufferVertex::desc(),
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
            depth_stencil: None,
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
            ab_glyph::FontArc::try_from_slice(include_bytes!("../assets/fonts/whitrabt.ttf"))
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
                // request a redraw if we got new info
                window.request_redraw();
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
                }
                winit::event::Event::RedrawRequested { .. } => {
                    // Get a command encoder for the current frame
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Redraw"),
                        });

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
                                depth_stencil_attachment: None,
                            });

                        render_pass.set_pipeline(&render_pipeline);
                        // add the texture bind group
                        render_pass.set_bind_group(0, &t_diffuse_bind_group, &[]);
                        // set the vertex buffer
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        // set the index buffer
                        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        // finally call draw but with indexed
                        render_pass.draw_indexed(0..num_indices, 0, 0..1);
                        //draw one instance of 3 vertices
                        // render_pass.draw(0..num_verts, 0..1);
                    }

                    glyph_brush.queue(Section {
                        screen_position: (30.0, 90.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(
                            format!("connected: {}", framedata.connected).as_str(),
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
    Ok(tx) // return the sender after we create the thread
}
