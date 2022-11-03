// make a global struct to store a copy of the in game info
// fill a batcher with operations to load from the fpga
// commit it
use log::{info, warn, Level, trace};
use memflow::prelude::{v1::*, memory_view::MemoryViewBatcher};

use ::std::ops::Add;

use crate::{offsets::*, datatypes::{tmp_vec2,tmp_vec3, game::WeaponId}};

pub mod entitylist;
use entitylist::{EntityList, EntityInfo};

#[derive(Debug)]
pub struct GameData {
    // Addresses
    pub client_state: Address,

    /// Local Player Info
    pub local_player: LocalPlayer,

    /// Entity List
    pub entity_list: EntityList,

    /// Temp Viewmatrix for reading into
    pub vm : [[f32;4];4],
    /// Local Player View Matrix
    pub view_matrix: glm::Mat4x4,
}

impl GameData {
    pub fn new(proc: &mut (impl Process + MemoryView), engine_base: Address, client_base: Address) -> Result<Self> {
        let client_state = proc.read_addr32(engine_base.add(*DW_CLIENTSTATE)).data()?;
        //let get_local_idx = proc.read::<u32>(client_state.add(*DW_CLIENTSTATE_GETLOCALPLAYER)).data()?;

        if !client_state.is_valid() || client_state.is_null() {
            return Err(Error(ErrorOrigin::Memory, ErrorKind::NotFound).log_error("client state address was not valid."));
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
                vm: Default::default(),
                view_matrix: Default::default(),
            };
        gd.load_data(proc, client_base)?;
        Ok(gd)
    }
    /// Load the data from the game in place using a batcher
    pub fn load_data(&mut self, proc: &mut (impl Process + MemoryView),client_base: Address) -> Result<()> {
        trace!("entering load data");

        // first update local player
        let local_player = proc.read_addr32(client_base.add(*DW_LOCALPLAYER)).data()?;

        if local_player.is_null() || !local_player.is_valid() {
            return Err(Error(ErrorOrigin::Memory, ErrorKind::NotFound).log_error("Local Player Address is not valid."));
        }

        self.local_player.address = local_player;
        
        let mut bat = proc.batcher();
        self.local_player.load_data(&mut bat, self.client_state);

        bat.read_into(client_base + *DW_VIEWMATRIX, &mut self.vm);

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
        let weapon_ptr = proc.read_addr32(client_base.add(*DW_ENTITYLIST + (self.local_player.weapon_ent_id-1) * 0x10)).data()?;
        let mut weapon_id:u32 = proc.read(weapon_ptr.add(*NET_ITEM_DEF_INDEX)).data()?;
        weapon_id &= 0xFFF;
        self.local_player.weapon_id = weapon_id.into();
        //println!("weapon id: {:?}", self.local_player.weapon_id);
        trace!("spec target: {} {} local: {}", self.local_player.observing_id, self.local_player.observing_id & 0xFFF, self.local_player.ent_idx);

        // retreive the entity list data:
        self.entity_list.populate_player_list(proc, client_base, &self.vm, self.local_player.ent_idx as usize)?;

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