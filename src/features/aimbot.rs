
use ::std::ops::Div;

use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::gamedata::GameData;
use crate::gamedata::entitylist::{tmp_vec2, tmp_vec3};
use crate::math;

pub fn aimbot(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>, game_data: &GameData) {
    if !kb.is_down(0x06) {return}
    //println!("velocity: {} vec: {:?}", game_data.local_player.vec_velocity.magnitude(),game_data.local_player.vec_velocity);
    //if game_data.local_player.vec_velocity.magnitude() > 1. {return}
    if let Some(closest_player) = game_data.entity_list.closest_player {

        let angles = game_data.local_player.view_angles - game_data.local_player.aimpunch_angle;
        println!("angle: {:?}",game_data.local_player.aimpunch_angle);

        // TODO: Store the distance_from_bone values in game_data to re use in both the triggerbot and in the aimbot
        // then make a nearest bone aimbot
        
        
    }

}

fn send_mouse_move(port: &mut Box<dyn SerialPort>, xin:i32,yin:i32) ->std::result::Result<usize, std::io::Error> {
    let cmd = b"mv"; // mouse move command
    let x = xin.to_le_bytes();// next 4 bytes
    let y = yin.to_le_bytes();// next 4 more bytes
    // combine these together in order and end with a newline byte
    let full_cmd: [u8;11] = [cmd[0],cmd[1],x[0],x[1],x[2],x[3],y[0],y[1],y[2],y[3],b"\n"[0]];
    port.write(&full_cmd)
}

