use ::std::{ops::{Add, IndexMut, Sub, Mul}, cell::RefCell, default};

use memflow::prelude::{v1::*, memory_view::MemoryViewBatcher};
use log::{info,trace};

use crate::{offsets::*, math};

#[repr(C)]
#[derive(Copy, Clone,Debug, Default, Pod)]
pub struct tmp_vec2 {
    pub x: f32,
    pub y: f32,
}

impl tmp_vec2 {
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.x.powf(2.)+self.y.powf(2.))
    }
    pub fn norm(&self, magnitude: f32) -> Self {
        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
        }
    }
}

impl Add for tmp_vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}
impl Add<tmp_vec3> for tmp_vec2 {
    type Output = tmp_vec3;

    fn add(self, rhs: tmp_vec3) -> Self::Output {
        tmp_vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: rhs.z
        }
    }
}

impl Sub for tmp_vec2 {
    type Output = Self;
    fn sub(self,rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl Sub<tmp_vec3> for tmp_vec2 {
    type Output = tmp_vec3;

    fn sub(self, rhs: tmp_vec3) -> Self::Output {
        tmp_vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: rhs.z
        }
    }
}

impl Mul<f32> for tmp_vec2 {
    type Output = Self;
    fn mul(self,rhs:f32) -> Self::Output {
        Self{
            x: self.x*rhs,
            y: self.y*rhs
        }
    }
}

impl From<glm::Vec2> for tmp_vec2 {
    fn from(other: glm::Vec2) -> Self {
        Self { x: other.x, y: other.y }
    }
}

#[repr(C)]
#[derive(Copy, Clone,Debug, Default, Pod)]
pub struct tmp_vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl tmp_vec3 {
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.x.powf(2.)+self.y.powf(2.)+self.z.powf(2.))
    }
    pub fn norm(&self, magnitude: f32) -> Self {
        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }
    /// Swizzling woohoo
    pub fn xy(&self) -> tmp_vec2 {
        tmp_vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl Add for tmp_vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}
impl Add<tmp_vec2> for tmp_vec3 {
    type Output = Self;

    fn add(self, rhs: tmp_vec2) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z
        }
    }
}

impl Sub for tmp_vec3 {
    type Output = Self;
    fn sub(self,rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl Sub<tmp_vec2> for tmp_vec3 {
    type Output = Self;

    fn sub(self, rhs: tmp_vec2) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z
        }
    }
}

impl Mul<f32> for tmp_vec3 {
    type Output = Self;
    fn mul(self,rhs:f32) -> Self::Output {
        Self{
            x: self.x*rhs,
            y: self.y*rhs,
            z: self.z*rhs
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

    pub screen_feet: Option<glm::Vec3>,
    pub screen_head: Option<glm::Vec3>,

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
            u32address: Default::default(),
            address: Default::default(),
            lifestate: Default::default(),
            health: Default::default(),
            team_num: Default::default(),
            vec_origin: Default::default(),
            vec_view_offset: Default::default(),
            vec_velocity: Default::default(),
            screen_feet: Default::default(),
            screen_head: Default::default(),
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
    pub entities: [EntityInfo; 64],
    pub closest_player: Option<usize>
}

impl Default for EntityList {
    fn default() -> EntityList {
        EntityList {
            entities: [EntityInfo::default(); 64],
            closest_player: None
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
    pub fn populate_player_list(&mut self, proc: &mut (impl Process + MemoryView), client_module_addr: Address, vm: &[[f32;4];4], local_player_idx: usize) -> Result<()> {
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
            // if ent.vec_view_offset.z == 0. {
            //     proc.read_into(ent.address.add(*NET_VEC_VIEWOFFSET), &mut ent.vec_view_offset).data()?;
            //     //ent.vec_view_offset.z = 64.06256;
            // }

            let feetpos = (ent.vec_origin).into();
            let headpos = (ent.head_pos).into();
            //if !math::is_world_point_visible_on_screen(&worldpos, &self.view_matrix) {continue}
            ent.screen_head = math::world_2_screen(
                &headpos,
                vm,
                None,
                None
            );
            ent.screen_feet = math::world_2_screen(
                &feetpos,
                vm,
                None,
                None
            );
            // set closest entity
            if let Some (head) = ent.screen_head {
                // only check for closest on visible entities
                if ent.spotted_by_mask & (1 << local_player_idx) > 0 {
                    let dist = glm::distance2(&head.xy(), &glm::vec2(1920./2.,1080./2.));
                    if self.closest_player.is_none() || dist < closest_dist.unwrap() {
                        closest_dist = Some(dist);
                        self.closest_player = Some(i);
                    }
                }
            }
            
        }

        trace!("exiting pop playerlist");
        Ok(())
    }
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