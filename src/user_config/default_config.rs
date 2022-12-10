use std::fmt;

use config::Map;
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
pub struct Trigger {
    pub visibility_check: bool,
    pub delay_ms: u32,
    pub max_inaccuracy: f32,
    pub max_velocity: f32,
}

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
pub struct AimBot {
    pub visibility_check: bool,
    pub delay_ms: u32,
}

#[derive(Serialize,Deserialize, Debug, Clone, Default)]
pub struct WeaponConfig {
    pub aimbot: AimBot,
    pub trigger: Trigger,
}

#[derive(Serialize,Deserialize, Debug, Clone)]
pub struct KeyBindings {
    pub trigger: i32,
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
                max_velocity: 90.,
            },
        });
        weapons.insert(WeaponId::Awp, WeaponConfig {
            aimbot: Default::default(),
            trigger: Trigger { 
                visibility_check: true,
                delay_ms: 5,
                max_inaccuracy: 0.045,
                max_velocity: 1.,
            },
        });
        weapons.insert(WeaponId::Mag7, WeaponConfig {
            aimbot: Default::default(),
            trigger: Trigger { 
                visibility_check: true,
                delay_ms: 0,
                max_inaccuracy: 8.,
                max_velocity: 380.,
            },
        });

        weapons.insert(WeaponId::Glock, WeaponConfig {
            aimbot: Default::default(),
            trigger: Trigger { 
                visibility_check: true,
                delay_ms: 0,
                max_inaccuracy: 0.7,
                max_velocity: 320.,
            },
        });

        weapons.insert(WeaponId::Usps, WeaponConfig {
            aimbot: Default::default(),
            trigger: Trigger { 
                visibility_check: true,
                delay_ms: 0,
                max_inaccuracy: 0.7,
                max_velocity: 200.,
            },
        });

        weapons.insert(WeaponId::Taser, WeaponConfig {
            aimbot: Default::default(),
            trigger: Trigger { 
                visibility_check: false,
                delay_ms: 0,
                max_inaccuracy: 99.,
                max_velocity: 400.,
            },
        });

        weapons.insert(WeaponId::Knife, WeaponConfig {
            aimbot: Default::default(),
            trigger: Trigger { 
                visibility_check: false,
                delay_ms: 0,
                max_inaccuracy: 99.,
                max_velocity: 400.,
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
                max_velocity: 80.,
            },
            weapons: WeaponConfigList(weapons),
        }
    }
}
