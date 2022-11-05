// This feature will record recoil per shotfired per gun type

use std::{error::Error, time::SystemTime, io::{Write, Read}, collections::{BTreeMap, HashMap}};

use log::{error, trace};
use serde::{Serialize,Deserialize};

use crate::{datatypes::{tmp_vec2, game::WeaponId, tmp_vec3}, gamedata::GameData, utils::math};

const FILENAME: &str = "recoil.json"; 

#[derive(Serialize,Deserialize, Default, Debug, Copy, Clone)]
pub struct RecoilData {
    pub angle: Option<tmp_vec2>,
    pub screen_pos: Option<tmp_vec2>,

    // acumulates angle movements until next shot
    #[serde(skip_serializing)]
    acumulator: Option<tmp_vec2>,

    // for recording real user recoil (diff = current_angle - last_angle)
    #[serde(skip_serializing)]
    last_angle: Option<tmp_vec2>,
}

#[derive(Serialize,Deserialize)]
struct GunRecoil {
    positions: Vec<Option<RecoilData>>,
}

// return a default capacity of 150 (the largest clip size in the game)
impl Default for GunRecoil {
    fn default() -> Self {
        let mut def = Self { positions: Vec::with_capacity(150) };
        // for i in 0..150 {
        //     def.positions.push(None);
        // }
        def
    }
}

pub struct RecoilRecorder {
    /// a list of gun recoil data which has recoil angle indexed by shotcount
    /// this vec is indexed by the WeaponId enum
    recoil_per_gun: HashMap<String,GunRecoil>,


    // some runtime vars we should not save

    last_save: SystemTime,
    old_punch: tmp_vec2,

    // for recording normal human spray
    start_angle: tmp_vec2,
}

impl RecoilRecorder {
    /// Try to load existing data from disk into memory and if failed initialize with a fresh instance
    pub fn new() -> Self {
        // TODO
        // for now just return Self
        let mut new = Self {
            recoil_per_gun: Default::default(),
            last_save: SystemTime::now(),
            old_punch: Default::default(),
            start_angle: Default::default(),
        };
        match new.load_data() {
            Err(e) => {
                error!("There was an error loading the recoil data from disk. {}", e);
            },
            _=>{}
        }
        new
    }

    // getters

    pub fn get_recoil_at(&self, shot: usize, for_weapon: WeaponId) -> Option<RecoilData> {
        if let Some(gun_data) = self.recoil_per_gun.get(&for_weapon.to_string()) {
            return *gun_data.positions.get(shot).unwrap_or(&None);
        }
        None
    }

    /// take in game_data each run and use it to insert recoil data into storage
    pub fn process_frame(&mut self, game_data: &GameData, legit_record: bool) {
        if game_data.local_player.lifestate != 0 {return}
        let sf = game_data.local_player.shots_fired as usize;
        let gun = game_data.local_player.weapon_id;
        let new_punch = game_data.local_player.aimpunch_angle *2.;
        let old_punch = self.old_punch;
        self.old_punch = new_punch;

        // get reference to the array of recoil angles for the currently heald weapon
        let gun_data = &mut self.recoil_per_gun.entry(gun.to_string()).or_insert_with(||Default::default());

        if sf == 0 {
            // set some kind of initial state in here before firing starts such as player starting angle
            self.old_punch = Default::default();
            if legit_record {
                self.start_angle = game_data.local_player.view_angles.xy();
                // reset the last angles
                for (i, gd) in gun_data.positions.iter_mut().enumerate() {
                    if let Some(recoildat) = gd {
                        recoildat.last_angle = None
                    }
                }
            } else {
                for (i, gd) in gun_data.positions.iter_mut().enumerate() {
                    if let Some(recoildat) = gd {
                        recoildat.acumulator = None
                    }
                }
            }
        } else if sf > 0 {
            
            // while shots fired is greater than size of recoil data array increase size of array
            while sf > gun_data.positions.len() { // for some stupid fucking reason this crap does not reserve properly
                trace!("reserving capacity for gundata positions");
                gun_data.positions.push(None);
            }

            // get a reference to the angle data for the current shot pos
            let (last,angle_storage_vec) = gun_data.positions.split_at_mut(sf-1);
            let angle_storage =  angle_storage_vec.first_mut().unwrap();

            let start_angle = if sf == 1 {
                self.start_angle
            } else {
                // get the angle from the last itteration
                last.first().unwrap().unwrap().last_angle.unwrap()
            };

            let mut to_add = 
            if legit_record {
                game_data.local_player.view_angles.xy() - self.start_angle
            } else {
                new_punch - old_punch
            };

            if let Some(storage) = angle_storage {
                if let Some(acum) = storage.acumulator {
                    to_add = (to_add + acum) / 2.;
                }
            }

            let recoil_angle =  
            if angle_storage.is_none() {
                to_add
            } else {
                let old_angle = angle_storage.unwrap().angle.unwrap();
                let avg_angle = (old_angle + to_add) / 2.;
                avg_angle
            };



            if let Some(recoil_screen) = recoil_angle_to_screen(game_data, recoil_angle) {
                if let Some(crosshair_screen) = recoil_angle_to_screen(game_data, Default::default()) {
                    let diff =  recoil_screen - crosshair_screen;
                    //println!("sf {} diff: {:?}", sf, diff);
                    // flip the direction of x
                    //diff.x = -diff.x;
                    *angle_storage = Some(RecoilData { angle: Some(recoil_angle), screen_pos: Some(diff), last_angle: Some(game_data.local_player.view_angles.xy()), acumulator: Some(to_add) });

                }
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

fn recoil_angle_to_screen(game_data: &GameData, recoil: tmp_vec2) -> Option<tmp_vec2> {
    let angles = game_data.local_player.view_angles;
    //let recoil = game_data.local_player.aimpunch_angle*2.;
    let recoil_world = math::get_crosshair_world_point_at_dist(
        200.,
        game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
        angles + recoil
    );
    if let Some(recoil_screen) = math::world_2_screen(
        &recoil_world.into(),
        &game_data.vm,
        None,
        None,
    ) {
        return Some(tmp_vec2 { x: recoil_screen.x, y: recoil_screen.y })
    }
    None
}