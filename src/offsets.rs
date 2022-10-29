use config::Config;
use lazy_static::lazy_static;
use std::sync::RwLock;
use log::info;

lazy_static! {
    /// The csgo offset config values
    static ref SETTINGS: RwLock<Config> = RwLock::new(init_offsets().unwrap());

    // Offsets

    pub static ref DW_CLIENTSTATE: u32 = load_offset("signatures.dwClientState");
    pub static ref DW_CLIENTSTATE_GETLOCALPLAYER: u32 = load_offset("signatures.dwClientState_GetLocalPlayer");
    pub static ref DW_CLIENTSTATE_VIEWANGLES: u32 = load_offset("signatures.dwClientState_ViewAngles");
    pub static ref DW_LOCALPLAYER: u32 = load_offset("signatures.dwLocalPlayer");
    pub static ref DW_ENTITYLIST: u32 = load_offset("signatures.dwEntityList");
    pub static ref DW_VIEWMATRIX: u32 = load_offset("signatures.dwViewMatrix");
    pub static ref M_BDORMANT: u32 = load_offset("signatures.m_bDormant");

    // Netvars

    pub static ref NET_HEALTH: u32 = load_offset("netvars.m_iHealth");
    pub static ref NET_CROSSHAIRID: u32 = load_offset("netvars.m_iCrosshairId");
    pub static ref NET_TEAM: u32 = load_offset("netvars.m_iTeamNum");
    pub static ref NET_LIFESTATE: u32 = load_offset("netvars.m_lifeState");
    pub static ref NET_SHOTSFIRED: u32 = load_offset("netvars.m_iShotsFired");
    pub static ref NET_AIMPUNCH_ANGLE: u32 = load_offset("netvars.m_aimPunchAngle");
    pub static ref NET_DW_BONEMATRIX: u32 = load_offset("netvars.m_dwBoneMatrix");
    pub static ref NET_OBSERVER_TARGET: u32 = load_offset("netvars.m_hObserverTarget");
    // viewpunch
    // aimpunch velocity
    pub static ref NET_VEC_ORIGIN: u32 = load_offset("netvars.m_vecOrigin");
    pub static ref NET_VEC_VIEWOFFSET: u32 = load_offset("netvars.m_vecViewOffset");
    pub static ref NET_VEC_VELOCITY: u32 = load_offset("netvars.m_vecVelocity");
}

// TODO: also add a source in here from a passed in config arg
fn init_offsets() -> std::result::Result<Config, Box<dyn std::error::Error>>{
    info!("initializing offsets config");
     let offsets = Config::builder()
    .add_source(config::File::with_name("hazedumper/csgo").required(false))
    .add_source(config::File::with_name("csgo").required(false))
    .build()?;
    Ok(offsets)
}

fn load_offset(key: &str) -> u32 {
    let offset = SETTINGS.read().expect("error getting read lock on settings").get::<u32>(key).expect(format!("could not find offset in config file for key {}", key).as_str());
    info!("loaded offset {}: {}", key, offset);
    offset
}