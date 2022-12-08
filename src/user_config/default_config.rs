use std::fmt;

use config::Config;
use log::info;
use serde::{Serialize, Deserialize};

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
pub enum CameraType {
    // Fully Zoomed out square that does not rotate
    #[default]
    Static,
    // Fully Zoomed out rotating around map center with player look direction
    Rotating,
    // Follows Player Location zoomed in and rotates with look direction
    RotatingFollow
}

// writes out the debug name as a string for normal formatting
// also allows .to_string() to be used
// https://stackoverflow.com/questions/32710187/how-do-i-get-an-enum-as-a-string
impl fmt::Display for CameraType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

impl From<String> for CameraType {
    fn from(v: String) -> Self {
        match v {
            x if x == CameraType::Static.to_string() => CameraType::Static,
            x if x == CameraType::Rotating.to_string() => CameraType::Rotating,
            x if x == CameraType::RotatingFollow.to_string() => CameraType::RotatingFollow,
            _ => CameraType::default(),
        }
    }
}


#[derive(Serialize,Deserialize, Debug, Clone, Default)]
struct Radar {
    enabled: bool,
    show_usernames: bool,
    camera_type: CameraType,
}

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
struct Trigger {
    enabled: bool,
    visibility_check: bool,
    delay_ms: u32,
    keybind: u32,
}

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
struct AimBot {
    enabled: bool,
    visibility_check: bool,
    delay_ms: u32,
}

#[derive(Serialize,Deserialize, Debug, Clone)]
pub struct DefaultConfig {
    radar: Radar,
    trigger: Trigger,
    aimbot: AimBot,
    bhop_enabled: bool,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self { 
            radar: Radar { enabled: true, show_usernames: true, camera_type: Default::default() },
            aimbot: Default::default(),
            trigger: Trigger { enabled: true, visibility_check: true, delay_ms: 0, keybind: 0x06 },
            bhop_enabled: false,
        }
    }
}
