use config::Config;
use lazy_static::lazy_static;
use std::sync::RwLock;
use log::info;

lazy_static! {
    /// The csgo offset config values
    static ref SETTINGS: RwLock<Config> = RwLock::new(init_offsets().unwrap());

    // Offsets

    pub static ref DW_CLIENTSTATE: u32 = load_offset("signatures.dwClientState").unwrap();
    pub static ref DW_CLIENTSTATE_GETLOCALPLAYER: u32 = load_offset("signatures.dwClientState_GetLocalPlayer").unwrap();
    pub static ref DW_CLIENTSTATE_VIEWANGLES: u32 = load_offset("signatures.dwClientState_ViewAngles").unwrap();
    pub static ref DW_LOCALPLAYER: u32 = load_offset("signatures.dwLocalPlayer").unwrap();
    pub static ref DW_ENTITYLIST: u32 = load_offset("signatures.dwEntityList").unwrap();
    pub static ref M_BDORMANT: u32 = load_offset("signatures.m_bDormant").unwrap();
    

    // Netvars

    pub static ref NET_HEALTH: u32 = load_offset("netvars.m_iHealth").unwrap();
    pub static ref NET_CROSSHAIRID: u32 = load_offset("netvars.m_iCrosshairId").unwrap();
    pub static ref NET_TEAM: u32 = load_offset("netvars.m_iTeamNum").unwrap();
    pub static ref NET_LIFESTATE: u32 = load_offset("netvars.m_lifeState").unwrap();
    pub static ref NET_SHOTSFIRED: u32 = load_offset("netvars.m_iShotsFired").unwrap();
    pub static ref NET_AIMPUNCH_ANGLE: u32 = load_offset("netvars.m_aimPunchAngle").unwrap();
    // viewpunch
    // aimpunch velocity
    //vec origin
    //vecViewOffset
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

fn load_offset(key: &str) -> std::result::Result<u32, Box<dyn std::error::Error>>{
    let offset = SETTINGS.read()?.get::<u32>(key)?;
    info!("loaded offset {}: {}", key, offset);
    Ok(offset)
}