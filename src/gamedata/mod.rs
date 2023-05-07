// make a global struct to store a copy of the in game info
// fill a batcher with operations to load from the fpga
// commit it
use log::{info, trace};
use memflow::prelude::{v1::*, memory_view::MemoryViewBatcher};

use ::std::{ops::Add, time::SystemTime, sync::mpsc};

use crate::{offsets::*, datatypes::{tmp_vec2,tmp_vec3, game::WeaponId}, render::MapData};

pub mod entitylist;
use entitylist::EntityList;

use self::minimap_info::MapInfo;

pub mod minimap_info;

#[derive(Debug)]
pub struct GameData {
    // Addresses
    pub client_state: Address,

    pub local_player_idx: i32,
    pub view_angles: tmp_vec3,

    /// Entity List
    pub entity_list: EntityList,

    // viewmatrix for using the games existing viewmatrix if desired
    pub vm : [[f32;4];4],

    /// The currently being played map name string
    pub current_map: Option<String>,
    /// the info on the current maps radar graphic such as scale and world pos
    pub current_map_info: Option<MapInfo>,
    
    map_tx: mpsc::Sender<MapData>,

    last_local_player_update: SystemTime,
    // for checking if a new map is loaded ocaisionally
    last_map_update: SystemTime,
    // last map before map update to check if it changes
    old_map_name: Option<String>,
}

impl GameData {
    pub fn new(proc: &mut (impl Process + MemoryView), engine_base: Address, client_base: Address, map_tx: mpsc::Sender<MapData>) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let client_state = proc.read_addr32(engine_base.add(*DW_CLIENTSTATE)).data()?;
        info!("current client state address (non relative): {:?}", client_state);
        //let get_local_idx = proc.read::<u32>(client_state.add(*DW_CLIENTSTATE_GETLOCALPLAYER)).data()?;

        if !client_state.is_valid() || client_state.is_null() {
            return Err(Error(ErrorOrigin::Memory, ErrorKind::NotFound).log_error("client state address was not valid."))?;
        }

        let mut gd =
            GameData {
            client_state,

            local_player_idx: 0,
            view_angles: Default::default(),
                
            entity_list: Default::default(),
            vm: Default::default(),
            current_map: None,
            current_map_info: None,

            // private for running lazy updates
            map_tx,
            last_local_player_update: SystemTime::UNIX_EPOCH,
            last_map_update: SystemTime::UNIX_EPOCH,
            old_map_name: None,
            };
        gd.load_data(proc, client_base)?;
        Ok(gd)
    }
    /// Load the data from the game in place using a batcher
    pub fn load_data(&mut self, proc: &mut (impl Process + MemoryView),client_base: Address) -> std::result::Result<(), Box<dyn std::error::Error>> {
        trace!("entering load data");


        /* moving this to the entitylist update loop
        // first update local player
        if let Ok(elap) = self.last_local_player_update.elapsed() {
            // check for update if its been 30 seconds or if its null
            // (it only changes between games. But it might read null if paged out)
            // tbh when we have a ui it may be better to just click a new game button
            // or maybe read a diff var to check if in a match or not
            if elap.as_secs() > 15 || self.entity_list.local_player.address.is_null() || !self.entity_list.local_player.address.is_valid() {
                let local_player = proc.read_addr32(client_base.add(*DW_LOCALPLAYER)).data()?;
                self.entity_list.local_player.address = local_player;
                self.last_local_player_update = SystemTime::now();
            }
        }

        if self.entity_list.local_player.address.is_null() || !self.entity_list.local_player.address.is_valid() {
            return Err(Error(ErrorOrigin::Memory, ErrorKind::NotFound).log_error("Local Player Address is not valid."))?;
        }

         */

        if let Ok(elap) = self.last_map_update.elapsed() {
            if elap.as_secs() > 30 {
                // read map name with max len of 32 as its unlikely amy map names go over that len
                self.current_map = proc.read_char_string_n(self.client_state.add(*DW_CLIENTSTATE_MAP), 32).data().map_or(None, |s| {
                    if s.len() > 0 {
                        Some(s)
                    } else {
                        None
                    }
                });
                if self.current_map != self.old_map_name {
                    info!("current map updated: {:?}", self.current_map);
                    self.old_map_name = self.current_map.clone();

                    // minimap info struct update
                    if let Some(name) = self.current_map.clone() {
                        self.current_map_info = minimap_info::load_map_info(name).map_or(None, |map_info|Some(map_info));
                        self.map_tx.send(MapData{
                            map_name: self.current_map.clone(),
                            map_details: self.current_map_info,
                        })?;
                    }
                }
            }
        }
        
        let mut bat = proc.batcher();
        //self.local_player.load_data(&mut bat, self.client_state);

        bat.read_into(client_base + *DW_VIEWMATRIX, &mut self.vm)
        .read_into(self.client_state.add(*DW_CLIENTSTATE_VIEWANGLES), &mut self.view_angles)
        .read_into(self.client_state.add(*DW_CLIENTSTATE_GETLOCALPLAYER), &mut self.local_player_idx);

        // finally, commit all the reads and writes at once:
        bat.commit_rw().data_part()?;
        // drop the batcher now that we are done with it
        std::mem::drop(bat);

        // retreive the entity list data:
        self.entity_list.populate_player_list(
            proc,
            client_base,
            self.client_state,
            self.local_player_idx as usize,
            self.view_angles,
            
        )?;

        if self.entity_list.local_player.address.is_null() {
            return Err(Error(ErrorOrigin::Memory, ErrorKind::NotFound).log_error("Local Player Address is not valid."))?;
        }

        trace!("exiting load data");
        Ok(())
    }
}