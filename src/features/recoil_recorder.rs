// This feature will record recoil per shotfired per gun type

use std::{error::Error, time::SystemTime, io::{Write, Read}, collections::{BTreeMap, HashMap}};

use log::{error, trace};
use serde::{Serialize,Deserialize};

use crate::{datatypes::{tmp_vec2, game::WeaponId}, gamedata::GameData};

const FILENAME: &str = "recoil.json"; 

#[derive(Serialize,Deserialize, Default, Copy,Clone)]
struct RecoilData {
    angle: Option<tmp_vec2>,
    screen_pos: Option<tmp_vec2>,
}

#[derive(Serialize,Deserialize)]
struct GunRecoil {
    positions: Vec<Option<RecoilData>>,
}

// return a default capacity of 150 (the largest clip size in the game)
impl Default for GunRecoil {
    fn default() -> Self {
        let mut def = Self { positions: Vec::with_capacity(150) };
        for i in 0..150 {
            def.positions.push(None);
        }
        def
    }
}

#[derive(Serialize,Deserialize)]
pub struct RecoilRecorder {
    /// a list of gun recoil data which has recoil angle indexed by shotcount
    /// this vec is indexed by the WeaponId enum
    recoil_per_gun: HashMap<String,GunRecoil>,


    // some runtime vars we should not save
    #[serde(skip_serializing)]
    last_save: SystemTime,
}

impl RecoilRecorder {
    /// Try to load existing data from disk into memory and if failed initialize with a fresh instance
    pub fn new() -> Self {
        // TODO
        // for now just return Self
        let mut new = Self {
            recoil_per_gun: Default::default(),
            last_save: SystemTime::now(),
        };
        match new.load_data() {
            Err(e) => {
                error!("There was an error loading the recoil data from disk. {}", e);
            },
            _=>{}
        }
        new
    }

    /// take in game_data each run and use it to insert recoil data into storage
    pub fn process_frame(&mut self, game_data: &GameData) {
        if game_data.local_player.lifestate != 0 {return}
        let sf = game_data.local_player.shots_fired as usize;
        let gun = game_data.local_player.weapon_id;
        let new_angle = game_data.local_player.aimpunch_angle *2.;
        if sf == 0 {
            // set some kind of initial state in here before firing starts such as player starting angle
        } else if sf > 0 {
            let gun_data = &mut self.recoil_per_gun.entry(gun.to_string()).or_insert_with(||Default::default());
            if sf > gun_data.positions.capacity() { // for some stupid fucking reason this crap does not reserve properly
                trace!("reserving capacity for gundata positions");
                gun_data.positions.reserve(sf - gun_data.positions.capacity() + 1);
            }
            let angle_storage =  &mut gun_data.positions[sf-1];
            if angle_storage.is_none() {
                // if no value is stored yet for this gun at this shot index then simply set
                *angle_storage = Some(RecoilData { angle: Some(new_angle), screen_pos: None });
            } else {
                // if a value already exists then average the two together
                let old_angle = angle_storage.unwrap().angle.unwrap();
                *angle_storage = Some(RecoilData { angle: Some((old_angle + new_angle) / 2.), screen_pos: None });
            }
        } 

        // now save the data to disk if it has been a while since last save
        if let Ok(elap) = self.last_save.elapsed() {
            // save once per min
            if elap.as_secs() > 60 {
                match self.save_data() {
                    Err(e) => {
                        error!("There was an error saving the recoil data to disk. {}", e);
                    },
                    _=>{}
                }
            }
        }
    }

    /// write out the recoil data into a disk representation
    fn save_data(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use std::fs::OpenOptions;
        self.last_save = SystemTime::now();

        let final_file_path = std::path::Path::new(FILENAME);
        let mut temp_file_path = std::env::temp_dir();
        temp_file_path.push(FILENAME);
        if !temp_file_path.exists() {
            std::fs::File::create(temp_file_path.as_path())?;
        }

        let json_str = serde_json::to_string_pretty(&self.recoil_per_gun)?;
        let mut file = OpenOptions::new()
            //.create_new(true) // for whatever reason this makes you unable to overwrite
            .write(true)
            .read(false)
            .truncate(true)
            .open(temp_file_path.as_path())?;
        file.write(json_str.as_bytes())?;
        std::fs::copy(temp_file_path.as_path(), final_file_path)?;
        Ok(())
    }

    fn load_data(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {

        // if the file doesn't exist we will let the saver create it later
        let file_path = std::path::Path::new(FILENAME);
        if !file_path.exists() {
            return Ok(())
        }

        use std::fs::OpenOptions;
        let file = OpenOptions::new()
            .write(false)
            .read(true)
            .open(FILENAME)?;
        
        let obj = serde_json::from_reader(
            file
        )?;
        self.recoil_per_gun = obj;
        Ok(())
    }
}