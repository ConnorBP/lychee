
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use log::info;

use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::{gamedata::GameData, human_interface::HumanInterface, utils::math::get_angle_from_crosshair, datatypes::game::WeaponId};
use crate::datatypes::{tmp_vec2,tmp_vec3};
use crate::utils::math;
use format_bytes::format_bytes;

pub struct AimBot {
    old_target: Option<usize>,
    got_new_target: bool,
    target_aquired_time: SystemTime,
    last_targeting_time: SystemTime,
    old_punch: tmp_vec2,
}

// TODO: move this over to a config
const new_target_delay_ms: u128 = 600;
const continue_spray_delay_ms: u128 = 900;
const move_speed: f32 = 10.;

impl AimBot {
    pub fn new() -> Self {
        Self {
            old_target: None,
            got_new_target: false,
            target_aquired_time: SystemTime::now(),
            last_targeting_time: SystemTime::now(),
            old_punch: Default::default(),
        }
    }
    pub fn aimbot(&mut self, kb: &mut Win32Keyboard<impl MemoryView>, human: &mut HumanInterface, game_data: &GameData) {        
        //if !kb.is_down(0x06) {return}
        if !kb.is_down(0x01) {
            // reset target when not using
            self.got_new_target = true;
            self.old_target = None;
            self.old_punch = Default::default();
            human.clear_goal();
            return;
        }

        if game_data.local_player.weapon_id == WeaponId::None {
            return;
        }

        //info!("velocity: {} vec: {:?}", game_data.local_player.vec_velocity.magnitude(),game_data.local_player.vec_velocity);
        //if game_data.local_player.vec_velocity.magnitude() > 1. {return}
        if let Some(closest_player) = game_data.entity_list.closest_player {


            if closest_player <= 0 {return}
            let ent = &game_data.entity_list.entities[closest_player];

            //println!("current target: {closest_player} lifestate: {} dormant: {} health: {}", ent.lifestate, ent.dormant &1, ent.health);

            if (ent.dormant &1 == 1)
            || ent.health <= 0
            || ent.lifestate > 0
            || ent.team_num == game_data.local_player.team_num
            || game_data.local_player.observing_id == 0 || closest_player == game_data.local_player.observing_id as usize -1
            //|| ent.spotted_by_mask & (1 << game_data.local_player.ent_idx) > 0
            {
                return
            }

            
            // let dist_angle = get_angle_from_crosshair(
            //     game_data.entity_list.entities[closest_player].head_pos,
            //     game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            //     game_data.local_player.view_angles.xy()
            // );
    
            // println!("angle: {:?} view: {:?}", dist_angle, game_data.local_player.view_angles.xy());
            // // my angle calc is wack apparently


            if let Some(old_target) = self.old_target {
                // if the old target is not the new one then we have a new target
                if old_target != closest_player {
                    // TODO: add target switching delay
                    self.got_new_target = true;
                    self.old_target = Some(closest_player);
                }
            } else {
                // if there was no old target it is the first target and a new target
                self.got_new_target = true;
                self.old_target = Some(closest_player);
            }
    
            // reset the targetfound time when there is a new target
            if self.got_new_target == true {
                self.target_aquired_time = SystemTime::now();
                //self.old_punch = Default::default();
                info!("GOT NEW TARGET");
                // reset newtarget var
                self.got_new_target = false
            }




            if game_data.local_player.shots_fired < 2 {return}
            // check remaining bullets
            //if game_data.local_player.clip
    
            let targeting: bool = 
            if let Ok(elap) = self.target_aquired_time.elapsed() {
                if elap.as_millis() < new_target_delay_ms {
                    false
                } else {
                    true
                }
            } else {
                false
            };
            
            // let skew = 
            // if let Ok(elap) = self.target_aquired_time.elapsed() {
            //     let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs_f32();
            //     let deviation = f32::min(elap.as_secs_f32()/2., 1.);
    
            //     tmp_vec3 {
            //         x: deviation * f32::cos(now*0.8),
            //         y: deviation * f32::sin(now*0.6),
            //         z: deviation/2. * f32::sin(now),
            //     }
            // } else {
            //     tmp_vec3 {
            //         x:0.,y:0.,z:0.
            //     }
            // };
            
            let angles = game_data.local_player.view_angles;
            let recoil = game_data.local_player.aimpunch_angle*2.;

            // TODO: make this standalone spray control when not targeting
            if !targeting {
                self.old_punch = recoil;
                return;
            }

            let dist_angle = get_angle_from_crosshair(
                game_data.entity_list.entities[closest_player].head_pos,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles.xy() + recoil
            );

            // max fov check
            if dist_angle.magnitude() > 20. {return}

            human.set_goal_angle(tmp_vec2 { x: -dist_angle.y, y: dist_angle.x });
            //human.mouse_move(tmp_vec2 { x: math::angle_to_mouse(-dist_angle.y) as f32, y: math::angle_to_mouse(dist_angle.x) as f32 }).expect("mouse move");

            //println!("aimpunch: {:?} angle: {:?}", game_data.local_player.aimpunch_angle, dist_angle);
            


            // // where the center of the screen is in world coords at enemy dist
            // let crosshair_world = get_crosshair_world_point_at_dist(
            //     10.,
            //     game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            //     angles
            // );
    
            // get where the crosshair is + aimpunch in world coords at the distance of the enemy
            // if in rcs mode (not targeting enemy yet) also remove the aimpunch of last frame to avoid drift
            // let recoil_world = 
            // if targeting {
            //     get_crosshair_world_point(
            //         game_data.entity_list.entities[closest_player].head_pos,
            //         game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            //         angles + recoil //- *old_punch
            //     )
            // } else {
            //     get_crosshair_world_point(
            //         game_data.entity_list.entities[closest_player].head_pos,
            //         game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            //         angles + recoil - self.old_punch
            //     )
            // };
            
            self.old_punch = recoil;
    
            // let target_angle: tmp_vec2 = 
            // if targeting {
            //     // move to target
            //     game_data.entity_list.entities[closest_player].head_pos + skew
            // } else {
            //     // move to target center screen
            //     crosshair_world + skew
            // };
        } 
    
    }
}

/// Take in a target world position, the players position, and what direction they are looking
/// Then return a world point in the direction the player is looking at the distance of the target position
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

/// Take in a distance, the players position, and what direction they are looking
/// Then return a world point in the direction the player is looking at the distance of the target position
fn get_crosshair_world_point_at_dist(to_dist: f32, our_pos: tmp_vec3, eye_ang: tmp_vec3) -> tmp_vec3 {
    // get direction vector for our view angles
    let eye_vec = math::angle_to_vec(eye_ang.x, eye_ang.y);
    // now that we have a direction vector (unit) and a magnitude
    // we can get the point along our look direction line with origin + dist*unit
    our_pos + eye_vec*to_dist
}

// method for sending mouse move via bit banging. Replaced with a more simple, slower but more reliable serial method
// fn send_mouse_move(port: &mut Box<dyn SerialPort>, xin:i32,yin:i32) ->std::result::Result<usize, std::io::Error> {
//     let cmd = b"mv"; // mouse move command
//     let x = xin.to_ne_bytes();// next 4 bytes
//     let y = yin.to_ne_bytes();// next 4 more bytes
//     // combine these together in order and end with a newline byte
//     let full_cmd: [u8;11] = [cmd[0],cmd[1],x[0],x[1],x[2],x[3],y[0],y[1],y[2],y[3],b"\n"[0]];
//     port.write(&full_cmd)
// }

