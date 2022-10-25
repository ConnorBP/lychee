// make a global struct to store a copy of the in game info
// fill a batcher with operations to load from the fpga
// commit it

// todo move all of the reading into here on a batcher

use log::{info, warn, Level, trace};
use memflow::prelude::{v1::*, memory_view::MemoryViewBatcher};
use memflow_win32::prelude::v1::*;
use patternscan::scan;
use serialport::SerialPort;
use std::io::Cursor;
use ::std::{ops::Add, time::Duration};

use config::Config;
use lazy_static::lazy_static;
use std::sync::RwLock;

use crate::offsets::*;

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
}

impl GameData {
    pub fn new(proc: &mut (impl Process + MemoryView), engine_base: Address, client_base: Address) -> Result<Self> {
        let client_state = proc.read_addr32(engine_base.add(*DW_CLIENTSTATE)).data()?;
        //if let Ok(local_player_idx) = process.read::<u32>(client_state.add(*DW_CLIENTSTATE_GETLOCALPLAYER)).data_part()

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
                    aimpunch_angle: 0.,
                },
                entity_list: EntityList::default(),
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
        self.local_player.load_data(&mut bat);
        // finally, commit all the reads and writes at once:
        bat.commit_rw().data_part()?;

        // drop the batcher now that we are done with it
        std::mem::drop(bat);

        // retreive the entity list data:

        self.entity_list.populate_player_list(proc, client_base)?;

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
    pub aimpunch_angle: f32,

}

impl LocalPlayer {
    fn load_data<'bat>(&'bat mut self, bat: &mut MemoryViewBatcher<'bat,impl Process + MemoryView>) {
        trace!("entering localplayer load data");
        //let health: i32 = process.read(local_player.add(*offsets::NET_HEALTH)).data()?;
        //if let Ok(incross) = process.read::<i32>(local_player.add(*offsets::NET_CROSSHAIRID)).data()
        bat
        .read_into(self.address.add(*crate::offsets::NET_HEALTH), &mut self.health)
        .read_into(self.address.add(*crate::offsets::NET_CROSSHAIRID), &mut self.incross)
        .read_into(self.address.add(*M_BDORMANT), &mut self.dormant)
        .read_into(self.address.add(*NET_TEAM), &mut self.team_num)
        .read_into(self.address.add(*NET_LIFESTATE), &mut self.lifestate)
        .read_into(self.address.add(*NET_AIMPUNCH_ANGLE), &mut self.aimpunch_angle);
        trace!("exiting localplayer load data");
    }
}