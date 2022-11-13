use ::std::{ops::Add, time::SystemTime};

use memflow::prelude::{v1::*, memory_view::MemoryViewBatcher};
use log::trace;

use crate::{offsets::*, utils::math, datatypes::{tmp_vec2,tmp_vec3}};

#[derive(Clone,Debug)]
#[repr(C)]
pub struct EntityInfo {
    u32address: u32,
    address: Address,
    pub dormant: u8,
    //b_is_local_player: bool,
    //is_enemy: bool,
    pub name: String,

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
    
    pub spotted_by_mask: u64,
}

impl Default for EntityInfo {
    fn default() -> EntityInfo {
        EntityInfo {
            dormant: 1,
            name: "".to_string(),
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
            spotted_by_mask: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct EntityList {
    pub entities: [EntityInfo; 32],// can be up to 64 (in theory) but we are gonna save some time with only reading 32
    pub closest_player: Option<usize>,

    last_name_refresh: SystemTime,
}

impl Default for EntityList {
    fn default() -> EntityList {
        EntityList {
            entities: Default::default(),// can be up to 64 (in theory) but we are gonna save some time with only reading 32
            closest_player: None,
            last_name_refresh: SystemTime::now(),
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
    pub fn populate_player_list(&mut self, proc: &mut (impl Process + MemoryView), client_module_addr: Address, client_state: Address, local_player_idx: usize, local_view_angles: tmp_vec3, local_eye_pos: tmp_vec3) -> Result<()> {
        trace!("entering pop playerlist");
        let mut bat1 = proc.batcher();
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {continue};
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
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {continue};
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
        trace!("running world to screen on entities");

        // get head positions
        let mut bat3 = proc.batcher();
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {continue};
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            let addr = Address::from(ent.bone_matrix);
            if !addr.is_valid() || addr.is_null() {continue}
            // read out bone pos 8 from the bone matrix address.
            // bat3.read_into(addr.add(0x30*8+0x0C), &mut ent.head_pos.x)
            //     .read_into(addr.add(0x30*8+0x1C), &mut ent.head_pos.y)
            //     .read_into(addr.add(0x30*8+0x2C), &mut ent.head_pos.z);
            load_bone_batch(&mut bat3, 8, addr, &mut ent.head_pos); // head bone
            load_bone_batch(&mut bat3, 7, addr, &mut ent.neck_pos); // neck bone
            load_bone_batch(&mut bat3, 6, addr, &mut ent.upper_body_pos); // upper chest bone
            load_bone_batch(&mut bat3, 5, addr, &mut ent.middle_body_pos); // middle body bone
            load_bone_batch(&mut bat3, 4, addr, &mut ent.lower_body_pos); // belly bone
            //load_bone_batch(&mut bat3, 0, addr, &mut ent.pelvis_pos); // pelvis bone
        }
        bat3.commit_rw().data_part()?;
        std::mem::drop(bat3);

        self.closest_player = None;
        let mut closest_dist = None;
        // get world2screen data
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if i == local_player_idx {continue};
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            
            // only check for closest on visible entities
            //if ent.spotted_by_mask & (1 << local_player_idx) > 0 {
                // need access to local player data to calculate distance
                let dist = math::get_dist_from_crosshair(ent.head_pos, local_eye_pos, local_view_angles.xy());
                if self.closest_player.is_none() || dist < closest_dist.unwrap() {
                    closest_dist = Some(dist);
                    self.closest_player = Some(i);
                }
            //}

            
        }

        if let Ok(elap) = self.last_name_refresh.elapsed() {
            // refresh every 30 seconds
            if elap.as_secs_f32() > 30. {
                self.update_entity_names(proc, client_state)?;
                self.last_name_refresh = SystemTime::now();
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
            if let Ok(name) = get_entity_name(proc,items,i) {
                ent.name = name;
                //println!("name: {}", ent.name);
            }
        }
        Ok(())
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
fn get_entity_name(proc: &mut (impl Process + MemoryView), items: Address, ent_idx: usize) -> Result<String> {
    let player_info_ptr = proc.read_addr32(items.add(0x28 + (ent_idx * 0x34))).data()?;
    if player_info_ptr.is_null() {return Ok("NO NAME".to_string())}
    let bytes = proc.read_raw(player_info_ptr.add(0x10), 32).data()?;
    Ok(std::str::from_utf8(bytes.as_bytes()).unwrap_or("NO NAME").to_string())
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