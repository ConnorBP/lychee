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

pub struct PlayerLoc {
    pub head_pos: Option<glm::Vec2>,
    pub feet_pos: Option<glm::Vec2>,
    pub team: i32,
}

#[derive(Default)]
pub struct FrameData {
    pub locations: Vec<PlayerLoc>,
}

pub fn start_window_render() -> Result<mpsc::Sender<FrameData>, crow::Error> {

    let (tx, rx) = mpsc::channel::<FrameData>();

    thread::spawn(|| {
        let event_loop = EventLoopExtWindows::new_any_thread();
        //let event_loop = EventLoop::new();
        let mut ctx = Context::new(WindowBuilder::new(), &event_loop).expect("couldn't build the window context");

        let ct_texture = Texture::load(&mut ctx, "./textures/ct.png").expect("couldn't find the ct player texture on the disk");
        let t_texture = Texture::load(&mut ctx, "./textures/t.png").expect("couldn't find the t player texture on the disk");

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
                        for (i,player) in framedata.locations.iter().enumerate() {
                            // crow seems to render from bottom left up instead of top left down so we flip it here
                            // TODO: replace hard coded 1080 with adaptive window res
                            if let Some(head_pos) = player.head_pos {
                                let posy = 1080 - head_pos.y as i32;

                                if let Some(feet_pos) = player.feet_pos {
                                    let feet_posy = 1080 - feet_pos.y as i32;
                                    ctx.debug_line(&mut surface, (head_pos.x as i32, posy), (feet_pos.x as i32, feet_posy), (0.5,0.3,0.8,0.8))
                                }

                                // 3 = ct 2 = t 1= spec maybe
                                if player.team == 3 {
                                    ctx.draw(&mut surface, &ct_texture, (head_pos.x as i32, posy), &DrawConfig::default());
                                } else {
                                    ctx.draw(&mut surface, &t_texture, (head_pos.x as i32, posy), &DrawConfig::default());
                                }
                            }
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