use config::Config;
use log::info;
use serde::{Serialize, Deserialize};

mod default_config;
pub mod config_watcher;

use default_config::DefaultConfig;



/// Loads the user config from a given location or initializes it if it does not exist
pub fn init_user_config(name: &str) -> std::result::Result<Config, Box<dyn std::error::Error>> {
    write_default_file(name)?;
    info!("Loading User Config {}", name);
    let config = Config::builder()
    .add_source(config::File::with_name(name).required(true))
    .build()?;
    Ok(config)
}

fn write_default_file(name: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    let pathname = format!("{}.json",name);
    let path = std::path::Path::new(&pathname);
    if !path.exists() {
        info!("Config file at {:?} doesn't exist, writing defaults.", path);
        let mut file = OpenOptions::new()
             .read(true)
             .write(true)
             .create(true)
             .open(path)?;
        //let mut s = Config::try_from(&DefaultConfig::default())?;
        file.write_all(
            serde_json::to_string_pretty(
                &DefaultConfig::default()
            )?.as_bytes()
        )?;
    }
    Ok(())
}
