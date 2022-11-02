
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use lazy_static::lazy_static;
use log::error;

use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::gamedata::GameData;
use crate::gamedata::entitylist::{tmp_vec2, tmp_vec3};
use crate::math;
use codepage_437::{CP437_CONTROL, ToCp437};
use format_bytes::format_bytes;


lazy_static!{
    // stores the last targeted entity so we can tell if a new one is found
    static ref OLD_TARGET: Mutex<Option<usize>> = Mutex::new(None);
    static ref GOT_NEW_TARGET: Mutex<bool> = Mutex::new(false);
    static ref TARGET_AQUIRED_TIME: Mutex<SystemTime> = Mutex::new(SystemTime::now());
    static ref OLD_PUNCH: Mutex<tmp_vec2> = Mutex::new(Default::default());
}

// TODO: move this over to a config
const new_target_delay_ms: u128 = 600;
const move_speed: f32 = 10.;

pub fn aimbot(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>, game_data: &GameData) {
    //if !kb.is_down(0x06) {return}
    if !kb.is_down(0x01) {
        // reset target when not using
        *GOT_NEW_TARGET.lock().unwrap() = true;
        return;
    }
    //info!("velocity: {} vec: {:?}", game_data.local_player.vec_velocity.magnitude(),game_data.local_player.vec_velocity);
    //if game_data.local_player.vec_velocity.magnitude() > 1. {return}
    if let Some(closest_player) = game_data.entity_list.closest_player {

        if let Ok(mut oldref) = OLD_TARGET.lock() {
            if let Some(old_target) = oldref.as_mut() {
                 // if the old target is not the new one then we have a new target
                 if *old_target != closest_player {
                     // TODO: add target switching delay
                     *GOT_NEW_TARGET.lock().unwrap() = true;
                     *old_target = closest_player;
                 }
            } else {
                 // if there was no old target it is the first target and a new target
                 *GOT_NEW_TARGET.lock().unwrap() = true;
                 *oldref = Some(closest_player);
            }
         } else {
             error!("failed to get mutex lock");
         }

        // reset the targetfound time when there is a new target
        let mut new_target = GOT_NEW_TARGET.lock().unwrap();
        let mut target_time = TARGET_AQUIRED_TIME.lock().unwrap();
         let mut old_punch = OLD_PUNCH.lock().unwrap();
        if *new_target == true {
            *target_time = SystemTime::now();
            *old_punch = Default::default();
            info!("GOT NEW TARGET");
            // reset newtarget var
            *new_target = false
        }

        if game_data.local_player.shots_fired < 2 {return}

        let targeting:bool = 
        if let Ok(elap) = (*target_time).elapsed() {
            if elap.as_millis() < new_target_delay_ms {
                false
            } else {
                true
            }
        } else {
            false
        };
        
        let skew = 
        if let Ok(elap) = (*target_time).elapsed() {
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f32();
            let deviation = f32::min(elap.as_secs_f32()/2., 1.);

            tmp_vec3 {
                x: deviation * f32::cos(now*0.8),
                y: deviation * f32::sin(now*0.6),
                z: deviation/2. * f32::sin(now),
            }
        } else {
            tmp_vec3 {
                x:0.,y:0.,z:0.
            }
        };
        
        let angles = game_data.local_player.view_angles;
        let recoil = game_data.local_player.aimpunch_angle*2.;
        info!("aimpunch: {:?}\nskew: {:?}", game_data.local_player.aimpunch_angle, skew);

        // where the center of the screen is in world coords at enemy dist
        let crosshair_world = get_crosshair_world_point(
            game_data.entity_list.entities[closest_player].head_pos,
            game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            angles
        );

        // get where the crosshair is + aimpunch in world coords at the distance of the enemy
        // if in rcs mode (not targeting enemy yet) also remove the aimpunch of last frame to avoid drift
        let recoil_world = 
        if targeting {
            get_crosshair_world_point(
                game_data.entity_list.entities[closest_player].head_pos,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles + recoil //- *old_punch
            )
        } else {
            get_crosshair_world_point(
                game_data.entity_list.entities[closest_player].head_pos,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles + recoil - *old_punch
            )
        };
        
        *old_punch = recoil;

        let target_pos:tmp_vec3 = 
        if targeting {
            // move to target
            game_data.entity_list.entities[closest_player].head_pos + skew
        } else {
            // move to target center screen
            crosshair_world + skew
        };
        

        // TODO: Store the distance_from_bone values in game_data to re use in both the triggerbot and in the aimbot
        // then make a nearest bone aimbot
        if let Some(target_screen) = math::world_2_screen(
            &target_pos.into(),
            &game_data.vm,
            None,
            None
        ) {
            if let Some(recoil_screen) = math::world_2_screen(
                &recoil_world.into(),
                &game_data.vm,
                None,
                None,
            ) {
                let diff: tmp_vec2 = tmp_vec2::from(target_screen.xy()) -  tmp_vec2::from(recoil_screen.xy());
                let direction = diff.norm(diff.magnitude()) * move_speed;


                //
                // move the mouse
                //
                info!("sending move x{} y{}", direction.x, direction.y);
                let x = direction.x as i32;
                let y = direction.y as i32;
                let cmd = format_bytes!(b"mv<{}><{}>\n", x,y);
                port.write(cmd.as_bytes()).expect("could not write to serial port");

                let mut serial_buf: Vec<u8> = vec![0; 200];
                if let Ok(t) = port.read(serial_buf.as_mut_slice()) {
                    std::io::stdout().write_all(&serial_buf[..t]).expect("failed to read serial");
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

