use config::{Config, File, Source};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use log::{info, debug};

mod findpattern;
pub use findpattern::*;

use self::hconfig::HConfig;
pub mod scanner;
pub mod hconfig;
pub mod output;
pub mod games;
pub mod helpers;

lazy_static! {
    /// The csgo offset config values
    static ref SETTINGS: RwLock<Config> = RwLock::new(init_offsets("csgo").unwrap());

    /// The hazedumper signature config file
    static ref SIG_CONFIG: RwLock<HConfig> = RwLock::new(load_hazed_config());
    
    //static ref SIGNATURES: RwLock<Config> = RwLock::new(init_offsets("config").unwrap());



    // Offsets

    pub static ref DW_DRAWOTHERMODELS: u32 = load_offset("signatures.dwDrawOtherModels");
    pub static ref DW_CLIENTSTATE: u32 = load_offset("signatures.dwClientState");
    pub static ref DW_CLIENTSTATE_GETLOCALPLAYER: u32 = load_offset("signatures.dwClientState_GetLocalPlayer");
    pub static ref DW_CLIENTSTATE_VIEWANGLES: u32 = load_offset("signatures.dwClientState_ViewAngles");
    pub static ref DW_CLIENTSTATE_MAP: u32 = load_offset("signatures.dwClientState_Map");
    pub static ref DW_CLIENTSTATE_PLAYERINFO: u32 = load_offset("signatures.dwClientState_PlayerInfo");
    pub static ref DW_LOCALPLAYER: u32 = load_offset("signatures.dwLocalPlayer");
    pub static ref DW_ENTITYLIST: u32 = load_offset("signatures.dwEntityList");
    pub static ref DW_RADARBASE: u32 = load_offset("signatures.dwRadarBase");
    pub static ref DW_VIEWMATRIX: u32 = load_offset("signatures.dwViewMatrix");
    pub static ref DW_FORCEJUMP: u32 = load_offset("signatures.dwForceJump");
    pub static ref M_BDORMANT: u32 = load_offset("signatures.m_bDormant");

    // Netvars

    pub static ref NET_HEALTH: u32 = load_offset("netvars.m_iHealth");
    pub static ref NET_CROSSHAIRID: u32 = load_offset("netvars.m_iCrosshairId");
    pub static ref NET_TEAM: u32 = load_offset("netvars.m_iTeamNum");
    pub static ref NET_LIFESTATE: u32 = load_offset("netvars.m_lifeState");
    pub static ref NET_SHOTSFIRED: u32 = load_offset("netvars.m_iShotsFired");
    pub static ref NET_AIMPUNCH_ANGLE: u32 = load_offset("netvars.m_aimPunchAngle");
    pub static ref NET_SHOTS_FIRED: u32 = load_offset("netvars.m_iShotsFired");
    pub static ref NET_DW_BONEMATRIX: u32 = load_offset("netvars.m_dwBoneMatrix");
    pub static ref NET_OBSERVER_TARGET: u32 = load_offset("netvars.m_hObserverTarget");
    pub static ref NET_ACTIVE_WEAPON: u32 = load_offset("netvars.m_hActiveWeapon");
    pub static ref NET_ITEM_DEF_INDEX: u32 = load_offset("netvars.m_iItemDefinitionIndex");
    pub static ref NET_SPOTTED_BY_MASK: u32 = load_offset("netvars.m_bSpottedByMask");
    pub static ref NET_VEC_ORIGIN: u32 = load_offset("netvars.m_vecOrigin");
    pub static ref NET_VEC_VIEWOFFSET: u32 = load_offset("netvars.m_vecViewOffset");
    pub static ref NET_VEC_VELOCITY: u32 = load_offset("netvars.m_vecVelocity");
    pub static ref NET_FLAGS: u32 = load_offset("netvars.m_fFlags");

}

/// Loads the hazedumper offsets from a config file of the given location
fn init_offsets(name: &str) -> std::result::Result<Config, Box<dyn std::error::Error>>{
    info!("initializing offsets config");
     let offsets = Config::builder()
    .add_source(config::File::with_name(format!("hazedumper/{name}").as_str()).required(true))
    //.add_source(config::File::with_name(name).required(false))
    .build()?;
    Ok(offsets)
}

fn load_offset(key: &str) -> u32 {
    let offset = SETTINGS.read().expect("error getting read lock on settings").get::<u32>(key).expect(format!("could not find offset in config file for key {}", key).as_str());
    info!("loaded offset {}: {}", key, offset);
    offset
}

// fn load_sig(key: &str) -> String {
//     let offset = SIGNATURES.read().expect("error getting read lock on settings").get_string(key).expect(format!("could not find offset in config file for key {}", key).as_str());
//     info!("loaded offset {}: {}", key, offset);
//     offset
// }

fn load_hazed_config() -> HConfig {
    let conf_path = "hazedumper/config.json";
    debug!("Loading config: {}", conf_path);
    hconfig::HConfig::load(&conf_path).expect("loading hazedumper signatures config")
}

// find dwSetClantag in engine.dll for testing
// pub fn test(data: &[u8]) {
//     let dw_set_clantag = "53 56 57 8B DA 8B F9 FF 15";
//     let index = find_pattern(data, dw_set_clantag);
//     println!("found {index:?}");
// }