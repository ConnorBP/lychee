use config::Config;
use lazy_static::lazy_static;
use std::sync::RwLock;
use log::info;

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new(init_offsets().unwrap());

    pub static ref DW_CLIENTSTATE: u32 = load_offset("signatures.dwClientState").unwrap();
    pub static ref DW_CLIENTSTATE_GETLOCALPLAYER: u32 = load_offset("signatures.dwClientState_GetLocalPlayer").unwrap();
    pub static ref DW_LOCALPLAYER: u32 = load_offset("signatures.dwLocalPlayer").unwrap();
    pub static ref DW_ENTITYLIST: u32 = load_offset("signatures.dwEntityList").unwrap();

    //netvars
    pub static ref NET_HEALTH: u32 = load_offset("netvars.m_iHealth").unwrap();
    pub static ref NET_CROSSHAIRID: u32 = load_offset("netvars.m_iCrosshairId").unwrap();
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