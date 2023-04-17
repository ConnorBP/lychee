use memflow::prelude::{Pod, v1::*};
use memflow_win32::prelude::v1::*;
use ::std::{ops::Add, time::SystemTime};
use crate::offsets::{find_pattern};

const BUFFER_MAX: usize = 256;


#[repr(C)]
#[derive(Pod)]
struct DXCOLOR {
    col: u32
}

#[repr(C)]
#[derive(Pod)]
struct BoxCommand {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    col: DXCOLOR,
}

#[repr(C)]
#[derive(Pod)]
struct BoxCommandBuffer {
    // we skip past the signature address of the struct (4 bytes)
    // signature: u32, // 0x4
    draw_count: u32,   // 0x8
    draw_ready: u32,   // 0x12
    reading: i32,      // 0x16

    // then buffer happens here
    //buffer: [BoxCommand;256]
}

pub struct Esp {
    buffer_addr: umem,

}

impl Esp {
    pub fn new(proc: &mut (impl Process + MemoryView)) -> Result<Self> {
        let buffer_magic = "0D F0 CC C0";
        // TODO: change name of dll
        let esp_module = proc.module_by_name("0x1337.dll")?;
        let dump = proc.read_raw(esp_module.base, esp_module.size as usize).data_part()?;
        let addr = find_pattern(&dump, buffer_magic).ok_or(Error(ErrorOrigin::Memory, ErrorKind::NotFound)/*.log_error("Failed to find ESP Buffer signature.")*/)? + 4;
        println!("*.* Found ESP Buffer Address: {addr:#02x}");

        Ok(
            Self {
                buffer_addr: addr as umem,
            }
        )
    }
}
