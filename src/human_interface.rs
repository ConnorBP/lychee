use std::time::{Duration, SystemTime};

use log::info;
use memflow::prelude::{PodMethods};
use serialport::SerialPort;
use format_bytes::format_bytes;

use crate::{datatypes::{tmp_vec2, tmp_vec3}, utils::math::angle_to_mouse};

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
    last_move: SystemTime,

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
            last_move: SystemTime::now(),

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
    pub fn mouse_move(&mut self, direction: tmp_vec2) -> std::result::Result<(), Box<dyn std::error::Error>> {
        //
        // move the mouse
        //
        info!("sending move x{} y{}", direction.x, direction.y);
        let mut x = direction.x.round() as i32;
        let mut y = direction.y.round() as i32;

        // cap at max i16 with no overflow
        if x > i16::MAX as i32 {x = i16::MAX as i32}
        if x < i16::MIN as i32 {x = i16::MIN as i32}
        if y > i16::MAX as i32 {y = i16::MAX as i32}
        if y < i16::MIN as i32 {y = i16::MIN as i32}

        
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
            self.goal_pos = Some(destination);
        }
    }

    /// replaces the goal pos
    #[allow(dead_code)]
    pub fn set_goal(&mut self, destination: tmp_vec2) {
        self.goal_pos = Some(destination);
    }

    #[allow(dead_code)]
    pub fn set_goal_angle(&mut self, destination_angle: tmp_vec2) {
        self.goal_pos = Some(tmp_vec2 {
            x: angle_to_mouse(destination_angle.x) as f32,
            y: angle_to_mouse(destination_angle.y) as f32,
        });
    }

    /// removes the goal (stops the mouse move)
    #[allow(dead_code)]
    pub fn clear_goal(&mut self) {
        self.goal_pos = None;
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

            // don't run faster than 1Khz
            if let Ok(elap) = self.last_rightclick.elapsed() {
                if elap.as_micros() < 1000 {
                    return Ok(())
                }
            } else {
                return Ok(())
            }
            self.last_move = SystemTime::now();

            let drift = drift();

            //let move_speed = 2. * drift.z; // todo make this configurable
            let distance = goal.magnitude();
            let direction = (goal/*+drift.xy()*/) /10.;//goal.norm(distance) * move_speed;
            
            // bypass smooth

            // finally reset goal pos
            // (so that if things don't set it next frame cause they wanna stop targeting it doesn't keep going)
            self.goal_pos = None;

            // then send the mouse move or return error
            self.mouse_move(direction)?;

        }
        Ok(())
    }
}

fn drift() -> tmp_vec3 {
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f64();
    tmp_vec3 {
        x: 0.5 * f64::cos(now*0.8) as f32 / 10.,
        y: 0.5 * f64::sin(now*0.6) as f32 / 50.,
        z: 1.4 + f64::sin(now) as f32 / 10.,
    }
}