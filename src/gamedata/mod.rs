// make a global struct to store a copy of the in game info
// fill a batcher with operations to load from the fpga
// commit it

// todo move all of the reading into here on a batcher

use log::{info, warn, Level};
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

#[derive(Debug)]
pub struct GameData {
    // Addresses
    pub client_state: Address,


    pub local_player: LocalPlayer,
}

impl GameData {
    pub fn new(proc: &mut (impl Process + MemoryView + Clone), engine_base: Address, client_base: Address) -> Result<Self> {
        let client_state = proc.read_addr32(engine_base.add(*DW_CLIENTSTATE)).data()?;
        let local_player_addr = proc.read_addr32(client_base.add(*DW_LOCALPLAYER)).data()?;


        let mut gd =
            GameData {
            client_state,
                local_player: LocalPlayer {
                    address: local_player_addr,
                    health: 0,
                    incross: 0,
                }
            };
        gd.load_data(proc.batcher())?;
        Ok(gd)
    }
    /// Load the data from the game in place using a batcher
    pub fn load_data<'bat>(&'bat mut self, mut bat: MemoryViewBatcher<'bat, impl Process + MemoryView>) -> Result<()> {
        self.local_player.load_data(&mut bat);

        // finally, commit all the reads and writes at once:
        bat.commit_rw().data_part()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LocalPlayer {
    pub address: Address,
    pub health: i32,
    pub incross: i32,
}

impl LocalPlayer {
    fn load_data<'bat>(&'bat mut self, bat: &mut MemoryViewBatcher<'bat,impl Process + MemoryView>) {
        //let health: i32 = process.read(local_player.add(*offsets::NET_HEALTH)).data()?;
        //if let Ok(incross) = process.read::<i32>(local_player.add(*offsets::NET_CROSSHAIRID)).data()
        bat
        .read_into(self.address.add(*crate::offsets::NET_HEALTH), &mut self.health)
        .read_into(self.address.add(*crate::offsets::NET_CROSSHAIRID), &mut self.incross);

    }
}