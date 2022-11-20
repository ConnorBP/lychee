use std::time::{Duration, SystemTime};

use log::info;
use memflow::prelude::{PodMethods};
use serialport::SerialPort;
use format_bytes::format_bytes;

use crate::datatypes::tmp_vec2;

const MOUSE_CLICK_DELAY_MS: u32 = 100;
const MOUSE_UNCLICK_DELAY_MS: u32 = 70;

/// Handles abstacting away the functions needed to output human interface controls such as mouse clicks moves or keyboard output
#[allow(dead_code)]
pub struct HumanInterface {
    /// the Serial Port Connection
    port: Box<dyn SerialPort>,

    // timers for making sure we don't spam the serial port
    last_leftclick: SystemTime,
    last_rightclick: SystemTime,

    left_clicked: bool,
    right_clicked: bool,

    // for humanized movement
    goal_pos: Option<tmp_vec2>,
}

impl HumanInterface {
    pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        // init the connection to the serial port for mouse and keyboard output
        info!("Fetching Serial Ports...");
        let ports = serialport::available_ports()?;
        for p in ports {
            info!("{}", p.port_name);
        }
        let port = serialport::new("COM9", 9_600)
            .timeout(Duration::from_millis(10))
            .open()?;
        
        // example usage for mouse left click:
        //port.write(b"ml\n")?;
        Ok(Self {
            port,
            last_leftclick: SystemTime::now(),
            last_rightclick: SystemTime::now(),

            left_clicked: false,
            right_clicked: false,

            goal_pos: None,
        })
    }
    pub fn mouse_left(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Ok(elap) = self.last_leftclick.elapsed() {
            if elap.as_millis() > MOUSE_CLICK_DELAY_MS.into() {
                self.last_leftclick = SystemTime::now();
                self.left_clicked = true;
                self.port.write(b"ml\n")?;
            }
        }
        Ok(())
    }
    pub fn mouse_right(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Ok(elap) = self.last_rightclick.elapsed() {
            if elap.as_millis() > MOUSE_CLICK_DELAY_MS.into() {
                self.last_rightclick = SystemTime::now();
                self.right_clicked = true;
                self.port.write(b"mr\n")?;
            }
        }
        Ok(())
    }
    #[allow(dead_code)]
    fn mouse_move(&mut self, direction: tmp_vec2) -> std::result::Result<(), Box<dyn std::error::Error>> {
        //
        // move the mouse
        //
        info!("sending move x{} y{}", direction.x, direction.y);
        let x = direction.x.round() as i32;
        let y = direction.y.round() as i32;
        let cmd = format_bytes!(b"mv<{}><{}>\n", x,y);
        self.port.write(cmd.as_bytes())?;

        // for debugging if sent serial data was valid
        // let mut serial_buf: Vec<u8> = vec![0; 200];
        // if let Ok(t) = port.read(serial_buf.as_mut_slice()) {
        //     std::io::stdout().write_all(&serial_buf[..t]).expect("failed to read serial");
        // }

        Ok(())
    }

    /// adds to the goal mouse direction for this frame
    #[allow(dead_code)]
    pub fn add_goal(&mut self, destination: tmp_vec2) {
        if let Some(goal) = &mut self.goal_pos {
            *goal = *goal + destination;
        } else {
            self.goal_pos = Some(destination)
        }
    }

    pub fn process_unclicks(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if self.left_clicked {
            if let Ok(elap) = self.last_leftclick.elapsed() {
                if elap.as_millis() > MOUSE_UNCLICK_DELAY_MS.into() {
                    self.left_clicked = false;
                    self.port.write(b"mlu\n")?;
                }
            }
        }
        if self.right_clicked {
            if let Ok(elap) = self.last_rightclick.elapsed() {
                if elap.as_millis() > MOUSE_UNCLICK_DELAY_MS.into() {
                    self.right_clicked = false;
                    self.port.write(b"mru\n")?;
                }
            }
        }
        Ok(())
    }

    /// moves smoothly in the direction of the final goal pos and resets goal after
    /// goal should be created each frame by various features such as recoil + seperate aim tracking
    /// then this function should be called last each frame
    #[allow(dead_code)]
    pub fn process_smooth_mouse(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // only run if a goal is set
        if let Some(goal) = self.goal_pos {

            let move_speed = 2.; // todo make this configurable
            let distance = goal.magnitude();
            let direction = goal.norm(distance) * move_speed;

            // finally reset goal pos
            // (so that if things don't set it next frame cause they wanna stop targeting it doesn't keep going)
            self.goal_pos = None;

            // then send the mouse move or return error
            self.mouse_move(direction)?;

        }
        Ok(())
    }
}