// make a global struct to store a copy of the in game info
// fill a batcher with operations to load from the fpga
// commit it
use log::{info, warn, Level, trace};
use memflow::prelude::{v1::*, memory_view::MemoryViewBatcher};

use ::std::{ops::Add, time::SystemTime, io::Read, sync::mpsc};

use crate::{offsets::*, datatypes::{tmp_vec2,tmp_vec3, game::WeaponId}, render::MapData};

pub mod entitylist;
use entitylist::{EntityList, EntityInfo};

use self::minimap_info::MapInfo;

pub mod minimap_info;

#[derive(Debug)]
pub struct GameData {
    // Addresses
    pub client_state: Address,

    /// Local Player Info
    pub local_player: LocalPlayer,

    /// Entity List
    pub entity_list: EntityList,

    // viewmatrix for using the games existing viewmatrix if desired
    //pub vm : [[f32;4];4],

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
        //let get_local_idx = proc.read::<u32>(client_state.add(*DW_CLIENTSTATE_GETLOCALPLAYER)).data()?;

        if !client_state.is_valid() || client_state.is_null() {
            return Err(Error(ErrorOrigin::Memory, ErrorKind::NotFound).log_error("client state address was not valid."))?;
        }

        let mut gd =
            GameData {
            client_state,
                local_player: LocalPlayer {
                    address: Address::null(), // this will be loaded in when gd.load_data is called
                    health: 0,
                    incross: 0,
                    dormant: 0,
                    lifestate: 0,
                    team_num: 0,
                    aimpunch_angle: Default::default(),
                    shots_fired: 0,
                    ent_idx: 0,
                    vec_origin: Default::default(),
                    vec_view_offset: Default::default(),
                    view_angles: Default::default(),
                    vec_velocity: Default::default(),
                    observing_id: 0,
                    weapon_ent_id: 0,
                    weapon_id: WeaponId::None,
                },
                entity_list: Default::default(),
                //vm: Default::default(),
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

        // first update local player
        if let Ok(elap) = self.last_local_player_update.elapsed() {
            // check for update if its been 30 seconds or if its null
            // (it only changes between games. But it might read null if paged out)
            // tbh when we have a ui it may be better to just click a new game button
            // or maybe read a diff var to check if in a match or not
            if elap.as_secs() > 15 || self.local_player.address.is_null() || !self.local_player.address.is_valid() {
                let local_player = proc.read_addr32(client_base.add(*DW_LOCALPLAYER)).data()?;
                self.local_player.address = local_player;
                self.last_local_player_update = SystemTime::now();
            }
        }
        if self.local_player.address.is_null() || !self.local_player.address.is_valid() {
            return Err(Error(ErrorOrigin::Memory, ErrorKind::NotFound).log_error("Local Player Address is not valid."))?;
        }

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
        self.local_player.load_data(&mut bat, self.client_state);

        //bat.read_into(client_base + *DW_VIEWMATRIX, &mut self.vm);

        // finally, commit all the reads and writes at once:
        bat.commit_rw().data_part()?;
        // drop the batcher now that we are done with it
        std::mem::drop(bat);

        // apply the bit mask to convert handles to an index
        self.local_player.observing_id &= 0xFFF;
        self.local_player.weapon_ent_id &= 0xFFF;
        //println!("weapon: {}", self.local_player.weapon_id);

        //DWORD pWeapon = mem->ReadMem<DWORD>(ClientDLL + dwEntityList + (pWeaponEnt - 1) * 0x10);
        //int id = mem->ReadMem<int>(pWeapon + m_iItemDefinitionIndex);
        //bat1.read_into(client_module_addr.add(*DW_ENTITYLIST + (i as u32 * 0x10)), &mut ent.u32address);
        if(self.local_player.weapon_ent_id == 0) {
            self.local_player.weapon_id = WeaponId::None;
        } else {
            let weapon_ptr = proc.read_addr32(client_base.add(*DW_ENTITYLIST + (self.local_player.weapon_ent_id-1) * 0x10)).data()?;
            let mut weapon_id:u32 = proc.read(weapon_ptr.add(*NET_ITEM_DEF_INDEX)).data()?;
            weapon_id &= 0xFFF;
            self.local_player.weapon_id = weapon_id.into();
        }
        
        //println!("weapon id: {:?}", self.local_player.weapon_id);
        trace!("spec target: {} {} local: {}", self.local_player.observing_id, self.local_player.observing_id & 0xFFF, self.local_player.ent_idx);

        // retreive the entity list data:
        self.entity_list.populate_player_list(
            proc,
            client_base,
            self.client_state,
            self.local_player.ent_idx as usize,
            self.local_player.view_angles,
            self.local_player.vec_origin + self.local_player.vec_view_offset
        )?;

        trace!("exiting load data");
        Ok(())
    }
}

#[derive(Debug)]
pub struct LocalPlayer {
    pub address: Address,
    pub incross: i32,

    pub dormant: u8,
    pub lifestate: i32,
    pub health: i32,
    pub team_num: i32,
    pub aimpunch_angle: tmp_vec2,
    pub shots_fired: i32,

    pub ent_idx: i32,
    pub observing_id: u64,
    pub weapon_ent_id: u32,
    pub weapon_id: WeaponId,

    pub vec_origin: tmp_vec3,
    pub vec_view_offset: tmp_vec3,
    pub view_angles: tmp_vec3,
    pub vec_velocity: tmp_vec3,
}

impl LocalPlayer {
    fn load_data<'bat>(&'bat mut self, bat: &mut MemoryViewBatcher<'bat,impl Process + MemoryView>, client_state: Address) {
        trace!("entering localplayer load data");
        //let health: i32 = process.read(local_player.add(*offsets::NET_HEALTH)).data()?;
        //if let Ok(incross) = process.read::<i32>(local_player.add(*offsets::NET_CROSSHAIRID)).data()
        bat
        .read_into(self.address.add(*NET_HEALTH), &mut self.health)
        .read_into(self.address.add(*NET_CROSSHAIRID), &mut self.incross)
        .read_into(self.address.add(*M_BDORMANT), &mut self.dormant)
        .read_into(self.address.add(*NET_TEAM), &mut self.team_num)
        .read_into(self.address.add(*NET_LIFESTATE), &mut self.lifestate)
        .read_into(self.address.add(*NET_AIMPUNCH_ANGLE), &mut self.aimpunch_angle)
        .read_into(self.address.add(*NET_SHOTSFIRED), &mut self.shots_fired)
        .read_into(self.address.add(*NET_VEC_ORIGIN), &mut self.vec_origin)
        .read_into(self.address.add(*NET_VEC_VIEWOFFSET), &mut self.vec_view_offset)
        .read_into(self.address.add(*NET_VEC_VELOCITY), &mut self.vec_velocity)
        .read_into(self.address.add(*NET_OBSERVER_TARGET), &mut self.observing_id)
        .read_into(self.address.add(*NET_ACTIVE_WEAPON), &mut self.weapon_ent_id)
        .read_into(client_state.add(*DW_CLIENTSTATE_VIEWANGLES), &mut self.view_angles)
        .read_into(client_state.add(*DW_CLIENTSTATE_GETLOCALPLAYER), &mut self.ent_idx);
        trace!("exiting localplayer load data");
    }
}