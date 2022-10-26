use crow::{
    glutin::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder, platform::windows::EventLoopExtWindows,
    },
    Context, DrawConfig, Texture,
};

use std::sync::mpsc;
use std::thread;

#[derive(Default)]
pub struct FrameData {
    pub locations: Vec<glm::Vec2>,
}

pub fn start_window_render() -> Result<mpsc::Sender<FrameData>, crow::Error> {

    let (tx, rx) = mpsc::channel::<FrameData>();

    thread::spawn(|| {
        let event_loop = EventLoopExtWindows::new_any_thread();
        //let event_loop = EventLoop::new();
        let mut ctx = Context::new(WindowBuilder::new(), &event_loop).expect("couldn't build the window context");

        let texture = Texture::load(&mut ctx, "./textures/ct.png").expect("couldn't find the player texture on the disk");

        // our frame data to be rendered (a list of player screen positions)
        let mut framedata = FrameData::default();

        event_loop.run(
            move |event: Event<()>, _window_target: _, control_flow: &mut ControlFlow|
            {
                // first update the frame data if it was received
                if let Ok(frame) = rx.try_recv() {
                    framedata = frame;
                }

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    Event::WindowEvent {
                        event: WindowEvent::Resized(s),
                        ..
                    } => {
                        // TODO: Send the main thread the new window size
                    },
                    Event::MainEventsCleared => ctx.window().request_redraw(),
                    Event::RedrawRequested(_) => {
                        let mut surface = ctx.surface();
                        let (w,h) = ctx.window_dimensions();
                        ctx.clear_color(&mut surface, (0.4, 0.4, 0.8, 1.0));
                        for (i,pos) in framedata.locations.iter().enumerate() {
                            // crow seems to render from bottom left up instead of top left down so we flip it here
                            // TODO: replace hard coded 1080 with adaptive window res
                            let posy = 1080 - pos.y as i32;
                            ctx.draw(&mut surface, &texture, (pos.x as i32, posy), &DrawConfig::default());
                        }
                        ctx.present(surface).unwrap();
                    }
                    _ => (),
                }
            }
        )
    });
    Ok(tx)
}