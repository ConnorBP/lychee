use std::time::{Duration, SystemTime};

use log::info;
use memflow::prelude::Pod;
use serialport::SerialPort;
use format_bytes::format_bytes;

use crate::datatypes::tmp_vec2;

const MOUSE_CLICK_DELAY_MS: u32 = 100;

/// Handles abstacting away the functions needed to output human interface controls such as mouse clicks moves or keyboard output
pub struct HumanInterface {
    /// the Serial Port Connection
    port: Box<dyn SerialPort>,

    // timers for making sure we don't spam the serial port
    last_leftclick: SystemTime,
    last_rightclick: SystemTime,
}

impl HumanInterface {
    pub fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        // init the connection to the serial port for mouse and keyboard output
        info!("Fetching Serial Ports...");
        let ports = serialport::available_ports()?;
        for p in ports {
            info!("{}", p.port_name);
        }
        let port = serialport::new("COM3", 115_200)
            .timeout(Duration::from_millis(10))
            .open()?;
        
        // example usage for mouse left click:
        //port.write(b"ml\n")?;
        Ok(Self {
            port,
            last_leftclick: SystemTime::now(),
            last_rightclick: SystemTime::now(),
        })
    }
    pub fn mouse_left(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Ok(elap) = self.last_leftclick.elapsed() {
            if elap.as_millis() > MOUSE_CLICK_DELAY_MS.into() {
                self.last_leftclick = SystemTime::now();
                self.port.write(b"ml\n")?;
            }
        }
        Ok(())
    }
    pub fn mouse_right(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        if let Ok(elap) = self.last_rightclick.elapsed() {
            if elap.as_millis() > MOUSE_CLICK_DELAY_MS.into() {
                self.last_rightclick = SystemTime::now();
                self.port.write(b"mr\n")?;
            }
        }
        Ok(())
    }
    fn mouse_move(&mut self, direction: tmp_vec2) -> std::result::Result<(), Box<dyn std::error::Error>> {
        //
        // move the mouse
        //
        info!("sending move x{} y{}", direction.x, direction.y);
        let x = direction.x as i32;
        let y = direction.y as i32;
        let cmd = format_bytes!(b"mv<{}><{}>\n", x,y);
        self.port.write(cmd.as_bytes())?;
        Ok(())
    }
}