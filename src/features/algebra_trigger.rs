use log::info;
use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::{utils::math,gamedata::GameData, datatypes::{tmp_vec3, game::WeaponId}};

use super::zuesknife;

const prefire_factor: f64 = 12.;

#[derive(Default)]
pub struct AlgebraTrigger {
    last_dist: f32,
    speed_avg: f32,
}

impl AlgebraTrigger {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn algebra_trigger(&mut self, kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>, game_data: &GameData, delta: f64) {
        if !kb.is_down(0x06) {return}
        if game_data.local_player.shots_fired > 1 {return}
        if game_data.local_player.aimpunch_angle.magnitude() > 0.1 {return} // force acuracy
        //info!("velocity: {} vec: {:?}", game_data.local_player.vec_velocity.magnitude(),game_data.local_player.vec_velocity);
        //if game_data.local_player.vec_velocity.magnitude() > 1. {return}
        if let Some(closest_player) = game_data.entity_list.closest_player {
    
            let angles = game_data.local_player.view_angles + (game_data.local_player.aimpunch_angle*2.);
            //info!("angle: {:?}",game_data.local_player.aimpunch_angle);
            
            //let dist_from_head = glm::distance(&point.into(), &to.into());
            if !zuesknife::zues_knife_bot(port,game_data,closest_player) {return}

            let entity = &game_data.entity_list.entities[closest_player];

            let vel = entity.vec_velocity *2.;
            let dist_from_head = get_dist_from_crosshair(
                entity.head_pos + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles
            );
    
            let speed = (dist_from_head - self.last_dist) as f64 * delta;
            //println!("speed {}", self.speed_avg);
            self.last_dist = dist_from_head;
    
            let dist_from_neck = get_dist_from_crosshair(
                entity.neck_pos + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles
            );
            let dist_from_body = get_dist_from_crosshair(
                entity.upper_body_pos + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles
            );
            let dist_from_middle = get_dist_from_crosshair(
                entity.middle_body_pos + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles
            );
            let dist_from_lower = get_dist_from_crosshair(
                entity.upper_body_pos + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles
            );

            // speed factor. Bring speed value from something like 0.1-0.01 to more like 1.0-0.5 ish
            // value of speed is negative when moving towards enemy and positive when moving away
            let sf = (speed * prefire_factor) as f32;
            self.speed_avg = (self.speed_avg + sf) / 2.;
            //info!("dist from body: {}", dist_from_body);
            if dist_from_head + self.speed_avg < 5.
            || dist_from_neck + self.speed_avg < 6.
            || dist_from_body + self.speed_avg < 7.
            || dist_from_middle + self.speed_avg < 7.
            || dist_from_lower + self.speed_avg < 7.
            {
                //game_data.local_player.incross = closest_player as i32;
                port.write(b"ml\n").unwrap();
            }
        }
    }
}

fn get_dist_from_crosshair(to_pos: tmp_vec3, our_pos: tmp_vec3, eye_ang: tmp_vec3) -> f32 {
    // difference
    let diff = to_pos - our_pos;
    // get direction vector for our view angles
    let eye_vec = math::angle_to_vec(eye_ang.x, eye_ang.y);
    // get the magnitide (distance) between to and from
    let dmag = diff.magnitude();
    //let unit_vec = diff.norm(dmag);
    //let dist = glm::distance(&to.into(), &from.into()); //mag is same as dist

    // now that we have a direction vector (unit) and a magnitude
    // we can get the point along our look direction line with origin + dist*unit
    let point = our_pos + eye_vec*dmag;
    glm::distance(&point.into(), &to_pos.into())
}