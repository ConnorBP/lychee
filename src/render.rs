use std::error::Error;
use wgpu::{CompositeAlphaMode, include_wgsl};
use wgpu_glyph::ab_glyph::Glyph;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::event_loop::EventLoopBuilder;
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::Fullscreen;

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

pub fn start_window_render() -> std::result::Result<mpsc::Sender<FrameData>, Box<dyn std::error::Error>> {

    let (tx, rx) = mpsc::channel::<FrameData>();

    /*
    put this in event loop:
    // first update the frame data if it was received
    if let Ok(frame) = rx.try_recv() {
        framedata = frame;
    }
    */

    thread::spawn(|| {

        // our frame data to be rendered (a list of player screen positions)
        let mut framedata = FrameData::default();

        let event_loop =
            EventLoopBuilder::new()
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
        let surface = unsafe { instance.create_surface(&window)};

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
                .await.expect("Request device")
        });

        // load the shader
        let shader = device.create_shader_module(include_wgsl!("../assets/shaders/shader.wgsl"));

        // create staging belt
        let mut staging_belt = wgpu::util::StagingBelt::new(1024);

        // prepare swap chain
        let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut size = window.inner_size();

        let render_pipeline_layout = 
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        // make render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shader Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
            primitive: wgpu::PrimitiveState{
                topology: wgpu::PrimitiveTopology::TriangleList, // TODO: change this to point list later
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false },
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
             }
        );

        // prepare the glyph_brush
        let white_rabbit = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../assets/fonts/whitrabt.ttf"
        )).expect("could not load font");

        let mut glyph_brush = GlyphBrushBuilder::using_font(white_rabbit)
            .build(&device, render_format);
        
        // render loop
        window.request_redraw();

        event_loop.run(move |event, _, control_flow| {
            // this is to make sure that resources are cleaned up properly.
            // Since event loop run never returns we need it to take ownership of resources
            let _ = &instance;

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
                },
                winit::event::Event::RedrawRequested {..} => {
                    // Get a command encoder for the current frame
                    let mut encoder = device.create_command_encoder(
                        &wgpu::CommandEncoderDescriptor {
                            label: Some("Redraw"),
                        },
                    );

                    // get the next frame
                    let frame = surface.get_current_texture().expect("get next frame");
                    let view = &frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                    // clear frame
                    {
                        let mut render_pass = encoder.begin_render_pass(
                            &wgpu::RenderPassDescriptor {
                                label: Some("Render pass"),
                                color_attachments: &[Some(
                                    // this is what @location(0) in the fragment shader targets
                                    wgpu::RenderPassColorAttachment {
                                        view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(
                                                wgpu::Color {
                                                    r:0.4,
                                                    g:0.4,
                                                    b:0.2,
                                                    a:1.0,
                                                },
                                            ),
                                            store: true,
                                        },
                                    },
                                )],
                                depth_stencil_attachment: None,
                            },
                        );

                        render_pass.set_pipeline(&render_pipeline);
                        //draw one instance of 3 vertices
                        render_pass.draw(0..3, 0..1);
                    }

                    glyph_brush.queue(Section {
                        screen_position: (30.0,90.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new(format!("connected: {}", framedata.connected).as_str()).with_color([1.0,1.0,1.0,1.0]).with_scale(40.0)],
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
                            size.height
                        )
                        .expect("Draw queued");
                    // submit the work
                    staging_belt.finish();
                    queue.submit(Some(encoder.finish()));
                    frame.present();
                    // recall unused staging buffers
                    staging_belt.recall();
                },
                _=> {
                    // for any other control flows do a wait
                    //*control_flow = winit::event_loop::ControlFlow::Wait;
                }
            }
        })
    });
    Ok(tx)
}