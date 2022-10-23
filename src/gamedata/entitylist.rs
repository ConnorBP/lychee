use ::std::{ops::{Add, IndexMut}, cell::RefCell, default};

use memflow::prelude::v1::*;
use log::{info,trace};

use crate::offsets::*;

#[repr(C)]
#[derive(Copy, Clone,Debug, Default, Pod)]
pub struct tmp_vec2 {
    x: f32,
    y: f32,
}

#[repr(C)]
#[derive(Copy, Clone,Debug, Default, Pod)]
pub struct tmp_vec3 {
    x: f32,
    y: f32,
    z: f32,
}


#[derive(Copy, Clone,Debug)]
#[repr(C)]
pub struct EntityInfo {
    u32address: u32,
    address: Address,
    pub dormant: u8,
    //b_is_local_player: bool,
    //is_enemy: bool,

    pub lifestate: i32,
    pub health: i32,
    pub team_num: i32,

    // vec_origin: tmp_vec3,
    // vec_view_offset: tmp_vec3,
    // view_angles: tmp_vec3,

    // vec_feet: tmp_vec2,
    // vec_head: tmp_vec2,
}

impl Default for EntityInfo {
    fn default() -> EntityInfo {
        EntityInfo { dormant: 1, u32address: Default::default(), address: Default::default(), lifestate: Default::default(), health: Default::default(), team_num: Default::default() }
    }
}

#[derive(Debug)]
pub struct EntityList {
    pub entities: [EntityInfo; 64],
}

impl Default for EntityList {
    fn default() -> EntityList {
        EntityList {
            entities: [EntityInfo::default(); 64],
        }
    }
}



impl EntityList {
    pub fn populate_player_list(&mut self, proc: &mut (impl Process + MemoryView), client_module_addr: Address) -> Result<()> {
        trace!("entering pop playerlist");
        let mut bat1 = proc.batcher();
        for (i, ent) in self.entities.iter_mut().enumerate() {
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
            trace!("converting u32 to address");
            ent.address = Address::from(ent.u32address);
            if ent.address.is_valid() && !ent.address.is_null() {
                // address is not null and is valid and read successfully so now read some netvars
                trace!("reading netvars");
                bat2.read_into(ent.address.add(*M_BDORMANT), &mut ent.dormant);
                bat2.read_into(ent.address.add(*NET_HEALTH), &mut ent.health);
                bat2.read_into(ent.address.add(*NET_TEAM), &mut ent.team_num);
                bat2.read_into(ent.address.add(*NET_LIFESTATE), &mut ent.lifestate);
                
            } else {
                ent.dormant = 1;
                continue;
            }
        }
        trace!("comitting second playerlist batcher");
        bat2.commit_rw().data_part()?;// tbh idk if its better to use data() or data_part() here
        trace!("done comitting second playerlist batcher");
        trace!("exiting pop playerlist");
        Ok(())
    }
}

// pub fn read_entity_addr_by_index(proc: &mut (impl Process + MemoryView), client_module_addr: Address, for_index: u32) -> Result<Address> {
//     let entity = proc.read_addr32(client_module_addr.add(*crate::offsets::DW_ENTITYLIST + (for_index * 0x10))).data()?;
//     info!("got entity: {:?} for index {}", entity, for_index);
//     Ok(entity)
// }

// pub fn yeet<P>(proc: &mut P) where P: Process + MemoryView {

// }