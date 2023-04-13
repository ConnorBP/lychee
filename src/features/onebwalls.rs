use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use ::std::{ops::Add, time::SystemTime};
use crate::offsets::DW_DRAWOTHERMODELS;

//#[cfg(feature = "walls")]
pub struct Walls {
    last_set_value: u8,
    last_set_at: SystemTime,
}

pub fn turn_on_walls(proc: &mut (impl Process + MemoryView), client_base: Address) -> Result<()> {
    let address = client_base.add(0x1D1066);
    let val = proc.read::<u8>(address).data()?;
    println!("got val {val} from {address:?}");
    //proc.write_raw(address, &[0x1u8]).data()?;
    Ok(())
}

//#[cfg(feature = "walls")]
impl Walls {
    pub fn new() -> Self {
        Self { last_set_value: 1, last_set_at: SystemTime::now() }
    }

    pub fn toggle_walls_button(&mut self, kb: &mut Win32Keyboard<impl MemoryView>, proc: &mut (impl Process + MemoryView), client_base: Address) -> Result<()> {
        
        if kb.is_down(0x50) { // p key
            if let Ok(elap) = self.last_set_at.elapsed() {
                if elap.as_millis() > 200 {
                    let address = client_base.add(0x1D1066);
                    let val = proc.read::<u8>(address).data()?;
                    println!("got val {val} from {address:?}");

                    
                    let new_value: u32 = if self.last_set_value == 1 { 2 } else { 1 };
                    proc.write::<u32>(client_base.add(*DW_DRAWOTHERMODELS), &new_value).data()?;
                    println!("Gonna write {:?} to {:?} offset {:?}", new_value, client_base.add(*DW_DRAWOTHERMODELS), *DW_DRAWOTHERMODELS);
                    self.last_set_at = SystemTime::now();
                }
            }
        }
        Ok(())
    }
}