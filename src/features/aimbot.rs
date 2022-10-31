
use ::std::borrow::Cow;
use ::std::io::Write;
use ::std::ops::Div;

use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::gamedata::GameData;
use crate::gamedata::entitylist::{tmp_vec2, tmp_vec3};
use crate::math;
use codepage_437::{CP437_CONTROL, ToCp437};
use format_bytes::format_bytes;

pub fn aimbot(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>, game_data: &GameData) {
    //if !kb.is_down(0x06) {return}
    //println!("velocity: {} vec: {:?}", game_data.local_player.vec_velocity.magnitude(),game_data.local_player.vec_velocity);
    //if game_data.local_player.vec_velocity.magnitude() > 1. {return}
    if let Some(closest_player) = game_data.entity_list.closest_player {

        let angles = game_data.local_player.view_angles - game_data.local_player.aimpunch_angle;
        println!("aimpunch: {:?}", game_data.local_player.aimpunch_angle);

        // get where the crosshair is + aimpunch in world coords at the distance of the enemy
        let crosshair_world = get_crosshair_world_point(
            game_data.entity_list.entities[closest_player].head_pos,
            game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            angles
        );

        // TODO: Store the distance_from_bone values in game_data to re use in both the triggerbot and in the aimbot
        // then make a nearest bone aimbot
        if let Some(player_screen) = math::world_2_screen(
            &game_data.entity_list.entities[closest_player].head_pos.into(),
            &game_data.vm,
            None,
            None
        ) {
            if let Some(crosshair_screen) = math::world_2_screen(
                &crosshair_world.into(),
                &game_data.vm,
                None,
                None,
            ) {
                let diff: tmp_vec2 = tmp_vec2::from(player_screen.xy()) -  tmp_vec2::from(crosshair_screen.xy());
                let direction = diff.norm(diff.magnitude()) * 10.;

                println!("sending move x{} y{}", direction.x, direction.y);

                // send_mouse_move(port, direction.x as i32, direction.y as i32)
                //     .expect("failed to communicate with microcontroller in mouse move");
                
                //let formatted = format!("mv<{}><{}>\n", direction.x,direction.y);
                //let cmd = formatted.to_cp437(&CP437_CONTROL).expect("failed to convert command to codepage");
                let x = direction.x as i32;
                let y = direction.y as i32;
                let cmd = format_bytes!(b"mv<{}><{}>\n", x,y);
                port.write(cmd.as_bytes()).expect("could not write to serial port");

                let mut serial_buf: Vec<u8> = vec![0; 200];
                if let Ok(t) = port.read(serial_buf.as_mut_slice()) {
                    std::io::stdout().write_all(&serial_buf[..t]);
                }
            }
        }
        
    }

}

fn int_to_cstr(i: i32) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();
    if i.signum() == -1 {
        v.push(b"-"[0]);
    }

    v
}

fn get_crosshair_world_point(to_pos: tmp_vec3, our_pos: tmp_vec3, eye_ang: tmp_vec3) -> tmp_vec3 {
    // difference
    let diff = to_pos - our_pos;
    // get direction vector for our view angles
    let eye_vec = math::angle_to_vec(eye_ang.x, eye_ang.y);
    // get the magnitide (distance) between to and from
    let dmag = diff.magnitude();

    // now that we have a direction vector (unit) and a magnitude
    // we can get the point along our look direction line with origin + dist*unit
    our_pos + eye_vec*dmag
}

fn send_mouse_move(port: &mut Box<dyn SerialPort>, xin:i32,yin:i32) ->std::result::Result<usize, std::io::Error> {
    let cmd = b"mv"; // mouse move command
    let x = xin.to_ne_bytes();// next 4 bytes
    let y = yin.to_ne_bytes();// next 4 more bytes
    // combine these together in order and end with a newline byte
    let full_cmd: [u8;11] = [cmd[0],cmd[1],x[0],x[1],x[2],x[3],y[0],y[1],y[2],y[3],b"\n"[0]];
    port.write(&full_cmd)
}

