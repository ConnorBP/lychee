use std::error::Error;
use wgpu::CompositeAlphaMode;
use wgpu_glyph::ab_glyph::Glyph;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::event_loop::EventLoopBuilder;
use winit::platform::windows::EventLoopBuilderExtWindows;

use std::sync::mpsc;
use std::thread;

pub struct PlayerLoc {
    pub head_pos: Option<glm::Vec3>,
    pub feet_pos: Option<glm::Vec3>,
    pub team: i32,
}

#[derive(Default)]
pub struct FrameData {
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
            .with_resizable(false)
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

        // create staging belt
        let mut staging_belt = wgpu::util::StagingBelt::new(1024);

        // prepare swap chain
        let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
        let mut size = window.inner_size();

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
                    // first update the frame data if it was received
                    if let Ok(frame) = rx.try_recv() {
                        framedata = frame;
                    }
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
                        let _ = encoder.begin_render_pass(
                            &wgpu::RenderPassDescriptor {
                                label: Some("Render pass"),
                                color_attachments: &[Some(
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
                    }

                    glyph_brush.queue(Section {
                        screen_position: (30.0,90.0),
                        bounds: (size.width as f32, size.height as f32),
                        text: vec![Text::new("hello wgpu_glyph~!").with_color([1.0,1.0,1.0,1.0]).with_scale(40.0)],
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
                    // first update the frame data if it was received
                    if let Ok(frame) = rx.try_recv() {
                        framedata = frame;
                    }
                    *control_flow = winit::event_loop::ControlFlow::Wait;
                }
            }
        })
    });
    Ok(tx)
}