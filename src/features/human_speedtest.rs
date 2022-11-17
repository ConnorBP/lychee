use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;

use crate::human_interface::HumanInterface;

/// for testing if the serial bridge has any noticable latency
/// Simply sends shoot command upon keypress detection
pub fn shoot_speed_test(kb: &mut Win32Keyboard<impl MemoryView>, human: &mut HumanInterface) {
    if !kb.is_down(0x4C) {return} // L key
    human.mouse_left().expect("sending mouse left");
}