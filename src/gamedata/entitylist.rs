use ::std::{ops::{Add, IndexMut}, cell::RefCell, default};

use memflow::prelude::v1::*;
use log::{info,trace};

use crate::{offsets::*, math};

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

impl Add for tmp_vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Into<glm::Vec2> for tmp_vec2 {
    fn into(self) -> glm::Vec2 {
        glm::vec2(self.x, self.y)
    }
}

impl Into<glm::Vec3> for tmp_vec3 {
    fn into(self) -> glm::Vec3 {
        glm::vec3(self.x,self.y,self.z)
    }
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

    pub vec_origin: tmp_vec3,
    pub vec_view_offset: tmp_vec3,
    pub vec_velocity: tmp_vec3,

    pub vec_feet: glm::Vec2,
    pub vec_head: glm::Vec2,
}

impl Default for EntityInfo {
    fn default() -> EntityInfo {
        EntityInfo {
            dormant: 1,
            u32address: Default::default(),
            address: Default::default(),
            lifestate: Default::default(),
            health: Default::default(),
            team_num: Default::default(),
            vec_origin: Default::default(),
            vec_view_offset: Default::default(),
            vec_velocity: Default::default(),
            vec_feet: Default::default(),
            vec_head: Default::default(),
        }
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
    pub fn populate_player_list(&mut self, proc: &mut (impl Process + MemoryView), client_module_addr: Address, vm: &[[f32;4];4]) -> Result<()> {
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
                bat2.read_into(ent.address.add(*M_BDORMANT), &mut ent.dormant)
                    .read_into(ent.address.add(*NET_HEALTH), &mut ent.health)
                    .read_into(ent.address.add(*NET_TEAM), &mut ent.team_num)
                    .read_into(ent.address.add(*NET_LIFESTATE), &mut ent.lifestate)
                    .read_into(ent.address.add(*NET_VEC_ORIGIN), &mut ent.vec_origin)
                    .read_into(ent.address.add(*NET_VEC_VIEWOFFSET), &mut ent.vec_view_offset)
                    .read_into(ent.address.add(*NET_VEC_VELOCITY), &mut ent.vec_velocity);
                
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
        // get world2screen data
        for (i, ent) in self.entities.iter_mut().enumerate() {
            if(ent.dormant &1 == 1) || ent.lifestate > 0 {continue}
            let feetpos = (ent.vec_origin).into();
            let headpos = (ent.vec_origin + ent.vec_view_offset).into();
            //if !math::is_world_point_visible_on_screen(&worldpos, &self.view_matrix) {continue}
            if let Some(screenpos) = math::world_2_screen(
                &headpos,
                vm,
                None,
                None
            ) {
                ent.vec_head = screenpos;
            }
            if let Some(screenpos) = math::world_2_screen(
                &feetpos,
                vm,
                None,
                None
            ) {
                ent.vec_feet = screenpos;
            }
        }

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