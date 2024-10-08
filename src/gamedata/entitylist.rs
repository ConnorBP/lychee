use ::std::{ops::Add, time::SystemTime};

use memflow::prelude::{v1::*, memory_view::MemoryViewBatcher};
use log::trace;

use crate::{offsets::*, utils::math, datatypes::{tmp_vec3, game::WeaponId, tmp_vec2}};

#[derive(Clone,Debug)]
#[repr(C)]
pub struct EntityInfo {
    u32address: u32,
    address: Address,
    pub dormant: u8,
    //b_is_local_player: bool,
    //is_enemy: bool,
    pub name: String,
    pub ent_info: Address,

    pub lifestate: i32,
    pub health: i32,
    pub team_num: i32,

    pub vec_origin: tmp_vec3,
    pub vec_view_offset: tmp_vec3,
    pub vec_velocity: tmp_vec3,

    pub bone_matrix: u32,//address
    pub head_pos: tmp_vec3,
    pub neck_pos: tmp_vec3,
    pub upper_body_pos: tmp_vec3,
    pub middle_body_pos: tmp_vec3,
    pub lower_body_pos: tmp_vec3,
    pub pelvis_pos: tmp_vec3,

    pub left_foot_pos: tmp_vec3,
    pub right_foot_pos: tmp_vec3,
    
    pub spotted_by_mask: u64,

    // pub visible: bool,
    // pub wall_intersect: tmp_vec3,
}

impl Default for EntityInfo {
    fn default() -> EntityInfo {
        EntityInfo {
            dormant: 1,
            name: "".to_string(),
            ent_info: Default::default(),
            u32address: Default::default(),
            address: Default::default(),
            lifestate: Default::default(),
            health: Default::default(),
            team_num: Default::default(),
            vec_origin: Default::default(),
            vec_view_offset: Default::default(),
            vec_velocity: Default::default(),
            bone_matrix: Default::default(),
            head_pos: Default::default(),
            neck_pos: Default::default(),
            upper_body_pos: Default::default(),
            middle_body_pos: Default::default(),
            lower_body_pos: Default::default(),
            pelvis_pos: Default::default(),
            left_foot_pos: Default::default(),
            right_foot_pos: Default::default(),
            spotted_by_mask: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct EntityList {
    /// Local Player Info
    pub local_player: LocalPlayer,

    pub entities: [EntityInfo; 32],// can be up to 64 (in theory) but we are gonna save some time with only reading 32
    pub closest_player: Option<usize>,

    last_name_refresh: SystemTime,
    pub names_just_updated: bool,
}

impl Default for EntityList {
    fn default() -> EntityList {
        EntityList {
            local_player: LocalPlayer {
                addr32: 0,
                address: Address::null(), // this will be loaded in when gd.load_data is called
                health: 0,
                incross: 0,
                dormant: 0,
                lifestate: 0,
                team_num: 0,
                aimpunch_angle: Default::default(),
                shots_fired: 0,
                vec_origin: Default::default(),
                vec_view_offset: Default::default(),
                vec_velocity: Default::default(),
                observing_id: 0,
                weapon_ent_id: 0,
                weapon_id: WeaponId::None,
            },
            entities: Default::default(),// can be up to 64 (in theory) but we are gonna save some time with only reading 32
            closest_player: None,
            last_name_refresh: SystemTime::now(),
            names_just_updated: false,
        }
    }
}



impl EntityList {

    // Getter Funcs
    // For retreiving data that was loaded in the data retreival step

    /// Get the team number for an entity index
    pub fn get_team_for(&self, idx: usize) -> Option<i32> {
        if self.entities[idx].dormant &1 == 1 { return None }
        Some(self.entities[idx].team_num)
    }

    //
    // data retreival
    //

    /// Takes in a reference to the game process and the client module base address and then walks the entity list tree
    /// Data retreived from this is stored into the EntityList struct this is called on
    pub fn populate_player_list(&mut self, proc: &mut (impl Process + MemoryView), client_module_addr: Address, client_state: Address, local_player_idx: usize, local_view_angles: tmp_vec3) -> Result<()> {
        trace!("entering pop playerlist");
        let mut bat1 = proc.batcher();
        bat1.read_into(client_module_addr.add(*DW_ENTITYLIST + (local_player_idx as u32 * 0x10)), &mut self.local_player.addr32);
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {         
                continue;
            };
            // clear the spot first so if there is an error reading it ends up not valid
            ent.u32address = 0;
            // add a u32 sized read at the expected adress for the entity address to be at
            bat1.read_into(client_module_addr.add(*DW_ENTITYLIST + (i as u32 * 0x10)), &mut ent.u32address);
        }
        

        trace!("comitting first playerlist batcher");
        bat1.commit_rw().data_part()?;

        trace!("done comitting first playerlist batcher");

        std::mem::drop(bat1);
        
        trace!("dropped first playerlist batcher");
        

        trace!("starting second playerlist batcher");
        let mut bat2 = proc.batcher();
        trace!("created second playerlist batcher");

        // load local player with the rest of the entities
        self.local_player.address = Address::from(self.local_player.addr32);
        if self.local_player.address.is_valid() {
            self.local_player.load_data(&mut bat2);
        }
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {                
                continue;
            };
            trace!("converting u32 to address");
            ent.address = Address::from(ent.u32address);
            if ent.address.is_valid() && !ent.address.is_null() {
                // address is not null and is valid and read successfully so now read some netvars
                trace!("reading netvars");
                bat2.read_into(ent.address.add(*M_BDORMANT), &mut ent.dormant)
                    .read_into(ent.address.add(*NET_HEALTH), &mut ent.health)
                    .read_into(ent.address.add(*NET_TEAM), &mut ent.team_num)
                    .read_into(ent.address.add(*NET_SPOTTED_BY_MASK), &mut ent.spotted_by_mask)
                    //.read_into(ent.address.add(*NET_VEC_VIEWOFFSET), &mut ent.vec_view_offset)
                    //.read_into(ent.address.add(*NET_VEC_VELOCITY), &mut ent.vec_velocity)
                    .read_into(ent.address.add(*NET_VEC_ORIGIN), &mut ent.vec_origin)
                    .read_into(ent.address.add(*NET_LIFESTATE), &mut ent.lifestate)
                    .read_into(ent.address.add(*NET_DW_BONEMATRIX), &mut ent.bone_matrix);
                
            } else {
                ent.dormant = 1;
                continue;
            }
        }
        trace!("comitting second playerlist batcher");
        bat2.commit_rw().data_part()?;// tbh idk if its better to use data() or data_part() here
        trace!("done comitting second playerlist batcher");
        std::mem::drop(bat2);


        let local_eye_pos = self.local_player.vec_origin + self.local_player.vec_view_offset;

        // apply the bit mask to convert handles to an index
        self.local_player.observing_id &= 0xFFF;
        self.local_player.weapon_ent_id &= 0xFFF;

        if self.local_player.weapon_ent_id == 0 {
            self.local_player.weapon_id = WeaponId::None;
        } else {
            let weapon_ptr = proc.read_addr32(client_module_addr.add(*DW_ENTITYLIST + (self.local_player.weapon_ent_id-1) * 0x10)).data()?;
            let mut weapon_id:u32 = proc.read(weapon_ptr.add(*NET_ITEM_DEF_INDEX)).data()?;
            weapon_id &= 0xFFF;
            self.local_player.weapon_id = weapon_id.into();
        }
        //println!("weapon id: {:?}", self.local_player.weapon_id);
        //println!("spec target: {} local: {} origin {:?}", self.local_player.observing_id, local_player_idx, self.local_player.vec_origin);



        trace!("running world to screen on entities");

        // get head positions
        let mut bat3 = proc.batcher();
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {
                continue;
            };
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {
                ent.dormant = 1;
                continue
            }

            //println!("not dormant: {i}, team: {}", ent.team_num);

            update_bones(&mut bat3, ent);
        }
        bat3.commit_rw().data_part()?;
        std::mem::drop(bat3);

        self.closest_player = None;
        let mut closest_dist = None;
        // get world2screen data
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {
                continue;
            };
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            
            const disable_vischeck: bool = true;
            // only check for closest on visible entities
            if disable_vischeck || (ent.spotted_by_mask & (1 << local_player_idx) > 0) {
                // need access to local player data to calculate distance
                let dist = math::get_dist_from_crosshair(ent.head_pos, local_eye_pos, (local_view_angles + self.local_player.aimpunch_angle*2.).xy());
                if self.closest_player.is_none() || dist < closest_dist.unwrap() {
                    closest_dist = Some(dist);
                    self.closest_player = Some(i);
                }
            }


            //vischeck test
            // let (visible, wall_intersect) = map_bsp.is_visible(
            //     self.local_player.vec_origin + self.local_player.vec_view_offset,
            //     ent.head_pos,
            // );

            // ent.visible = visible;
            // ent.wall_intersect = wall_intersect;

            
        }

        if let Ok(elap) = self.last_name_refresh.elapsed() {
            // refresh every 30 seconds
            if elap.as_secs_f32() > 30. {
                self.update_entity_names(proc, client_state)?;
                self.last_name_refresh = SystemTime::now();
                self.names_just_updated = true;
            } else {
                self.names_just_updated = false;
            }
        }

        trace!("exiting pop playerlist");
        Ok(())
    }

    fn update_entity_names(&mut self, proc: &mut (impl Process + MemoryView), client_state: Address) -> Result<()> {
        let table = proc.read_addr32(client_state.add(*DW_CLIENTSTATE_PLAYERINFO)).data()?;
        if table.is_null() {return Ok(())}
        let items_ptr: Address = proc.read_addr32(table.add(0x40)).data()?.add(0xC);
        if items_ptr.is_null() {return Ok(())}
        let items = proc.read_addr32(items_ptr).data()?;
        if items.is_null() {return Ok(())}
        
        for (i, ent) in self.entities.iter_mut().enumerate() {
            //if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            if let Ok((info, name)) = get_entity_name(proc,items,i) {
                ent.name = name;
                ent.ent_info = info;
                //println!("name: {}", ent.name);
            }
        }
        Ok(())
    }
}


#[derive(Debug)]
pub struct LocalPlayer {
    addr32: u32,
    pub address: Address,
    pub incross: i32,

    pub dormant: u8,
    pub lifestate: i32,
    pub health: i32,
    pub team_num: i32,
    pub aimpunch_angle: tmp_vec2,
    pub shots_fired: i32,
    pub observing_id: u64,
    pub weapon_ent_id: u32,
    pub weapon_id: WeaponId,

    pub vec_origin: tmp_vec3,
    pub vec_view_offset: tmp_vec3,
    pub vec_velocity: tmp_vec3,
}

impl LocalPlayer {
    fn load_data<'bat>(&'bat mut self, bat: &mut MemoryViewBatcher<'bat,impl Process + MemoryView>) {
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
        .read_into(self.address.add(*NET_ACTIVE_WEAPON), &mut self.weapon_ent_id);
        trace!("exiting localplayer load data");
    }
}

// Do not use this recursively
// some example code of all the steps needed to read a player name from the playerinfo list
// fn read_single_entity_name(proc: &mut (impl Process + MemoryView), client_state: Address, ent_idx: usize) -> Result<String> {
//     let table = proc.read_addr32(client_state.add(*DW_CLIENTSTATE_PLAYERINFO)).data()?;
//     let items_ptr = proc.read_addr32(table.add(0x40)).data()?.add(0xC);
//     let items = proc.read_addr32(items_ptr).data()?;
//     let player_info_ptr = proc.read_addr32(items.add(0x28 + (ent_idx * 0x34))).data()?;
//     let bytes = proc.read_raw(player_info_ptr.add(0x10), 32).data()?;
//     Ok(std::str::from_utf8(bytes.as_bytes()).unwrap_or("NO NAME").to_string())
// }

/// given the pointer to the player info items list read out an entity username
fn get_entity_name(proc: &mut (impl Process + MemoryView), items: Address, ent_idx: usize) -> Result<(Address, String)> {
    let player_info_ptr = proc.read_addr32(items.add(0x28 + (ent_idx * 0x34))).data()?;
    if player_info_ptr.is_null() {return Ok((Address::NULL,"NO NAME".to_string()))}
    let bytes = proc.read_raw(player_info_ptr.add(0x10), 32).data()?;
    Ok((player_info_ptr, std::str::from_utf8(bytes.as_bytes()).unwrap_or("NO NAME").to_string()))
}

pub fn update_bones<'bat>(bat: &mut MemoryViewBatcher<'bat,impl Process + MemoryView>, out: &'bat mut EntityInfo) {
    let addr = Address::from(out.bone_matrix);
    if !addr.is_valid() || addr.is_null() {return}
    // read out bone pos 8 from the bone matrix address.
    // bat3.read_into(addr.add(0x30*8+0x0C), &mut ent.head_pos.x)
    //     .read_into(addr.add(0x30*8+0x1C), &mut ent.head_pos.y)
    //     .read_into(addr.add(0x30*8+0x2C), &mut ent.head_pos.z);
    load_bone_batch(bat, 8, addr, &mut out.head_pos); // head bone
    load_bone_batch(bat, 7, addr, &mut out.neck_pos); // neck bone
    load_bone_batch(bat, 6, addr, &mut out.upper_body_pos); // upper chest bone
    load_bone_batch(bat, 5, addr, &mut out.middle_body_pos); // middle body bone
    load_bone_batch(bat, 4, addr, &mut out.lower_body_pos); // belly bone
    //load_bone_batch(&mut bat3, 0, addr, &mut ent.pelvis_pos); // pelvis bone
    // feet are 79 and two https://www.unknowncheats.me/forum/counterstrike-global-offensive/195653-csgo-bone-id.html
    load_bone_batch(bat, 72, addr, &mut out.left_foot_pos);
    load_bone_batch(bat, 79, addr, &mut out.right_foot_pos);
}

fn load_bone_batch<'bat>(bat: &mut MemoryViewBatcher<'bat,impl Process + MemoryView>, bone_id: i32, bone_matrix: Address, out: &'bat mut tmp_vec3) {
    bat.read_into(bone_matrix.add(0x30*bone_id+0x0C), &mut out.x)
                .read_into(bone_matrix.add(0x30*bone_id+0x1C), &mut out.y)
                .read_into(bone_matrix.add(0x30*bone_id+0x2C), &mut out.z);
}

// pub fn read_entity_addr_by_index(proc: &mut (impl Process + MemoryView), client_module_addr: Address, for_index: u32) -> Result<Address> {
//     let entity = proc.read_addr32(client_module_addr.add(*crate::offsets::DW_ENTITYLIST + (for_index * 0x10))).data()?;
//     info!("got entity: {:?} for index {}", entity, for_index);
//     Ok(entity)
// }

// pub fn yeet<P>(proc: &mut P) where P: Process + MemoryView {

// }