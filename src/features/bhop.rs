use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;

pub fn bhop(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>) {
    if kb.is_down(0x20) {
        println!("space down");
        port.write(b"su\n").expect("could not write to serial");
    }
}