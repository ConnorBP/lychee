#[cfg(any(feature = "bhop_sus", feature = "bhop"))]
use memflow::prelude::v1::*;
#[cfg(any(feature = "bhop_sus", feature = "bhop"))]
use memflow_win32::prelude::v1::*;
#[cfg(feature = "bhop")]
use serialport::SerialPort;

#[cfg(feature = "bhop_sus")]
use ::std::{ops::Add, time::SystemTime};
#[cfg(feature = "bhop_sus")]
use crate::{gamedata::GameData, offsets::{NET_FLAGS, DW_FORCEJUMP}};

#[cfg(feature = "bhop")]
pub fn bhop(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>) {
    if kb.is_down(0x20) {
        port.write(b"su\n").expect("could not write to serial");
    }
}

#[cfg(feature = "bhop_sus")]
pub struct SusBhop {
    needs_reset: bool,
    last_shot: SystemTime,
}

#[cfg(feature = "bhop_sus")]
impl SusBhop {
    pub fn new() -> Self {
        Self { needs_reset: false, last_shot: SystemTime::now() }
    }
    pub fn bhop_sus(&mut self, kb: &mut Win32Keyboard<impl MemoryView>, proc: &mut (impl Process + MemoryView),game_data: &GameData, client_base: Address) -> Result<()> {
        if self.needs_reset {
            if let Ok(elap) = self.last_shot.elapsed() {
                if elap.as_millis() > 50 {
                    proc.write(client_base.add(*DW_FORCEJUMP), &0x4u8).data()?;
                    self.needs_reset = false;
                }
            }
            return Ok(());
        }
        if kb.is_down(0x20) {
            let flags: u64 = proc.read(game_data.local_player.address.add(*NET_FLAGS)).data().unwrap_or(0);
            let on_ground: bool = flags & 1 > 0;
            if on_ground {
                proc.write(client_base.add(*DW_FORCEJUMP), &0x5u8).data()?;
                self.needs_reset = true;
                self.last_shot = SystemTime::now();
            }
        }
        Ok(())
    }
}