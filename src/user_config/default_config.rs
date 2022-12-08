use std::fmt;

use config::{Config, Map};
use log::info;
use serde::{Serialize, Deserialize, Serializer, ser::SerializeMap};

use crate::datatypes::game::WeaponId;

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
    visibility_check: bool,
    delay_ms: u32,
    max_inaccuracy: f32,
    max_velocity: f32,
}

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
struct AimBot {
    visibility_check: bool,
    delay_ms: u32,
}

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
struct WeaponConfig {
    aimbot: AimBot,
    trigger: Trigger,
}

#[derive(Serialize,Deserialize, Debug, Clone)]
struct KeyBindings {
    trigger: u32,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self { trigger: 0x06 }
    }
}

#[derive(Debug, Clone)]
struct WeaponConfigList(Map<WeaponId, WeaponConfig>);

#[derive(Serialize, Debug, Clone)]
pub struct DefaultConfig {
    keybinds: KeyBindings,
    radar: Radar,
    bhop_enabled: bool,
    trigger_enabled: bool,
    trigger_defaults: Trigger,
    aimbot_enabled: bool,
    aimbot_defaults: AimBot,
    weapons: WeaponConfigList,
}

impl Serialize for WeaponConfigList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(&k.to_string(), &v)?;
        }
        map.end()
    }
}

impl Default for DefaultConfig {
    fn default() -> Self {

        let mut weapons = Map::new();
        weapons.insert(WeaponId::Ak47, WeaponConfig {
            aimbot: Default::default(),
            trigger: Trigger { 
                visibility_check: true,
                delay_ms: 0,
                max_inaccuracy: 0.065,
                max_velocity: 1.,
            },
        });

        Self { 
            keybinds: Default::default(),
            radar: Radar { enabled: true, show_usernames: true, camera_type: Default::default() },
            bhop_enabled: false,
            aimbot_enabled: false,
            trigger_enabled: true,
            aimbot_defaults: Default::default(),
            trigger_defaults: Trigger { 
                visibility_check: true,
                delay_ms: 0,
                max_inaccuracy: 0.065,
                max_velocity: 5.,
            },
            weapons: WeaponConfigList(weapons),
        }
    }
}
