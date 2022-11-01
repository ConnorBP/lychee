use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::gamedata::GameData;
use crate::gamedata::entitylist::{tmp_vec2, tmp_vec3};
use crate::math;

pub fn algebra_trigger(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>, game_data: &GameData) {
    if !kb.is_down(0x06) {return}
    if game_data.local_player.shots_fired > 1 {return}
    //println!("velocity: {} vec: {:?}", game_data.local_player.vec_velocity.magnitude(),game_data.local_player.vec_velocity);
    //if game_data.local_player.vec_velocity.magnitude() > 1. {return}
    if let Some(closest_player) = game_data.entity_list.closest_player {

        let angles = game_data.local_player.view_angles + (game_data.local_player.aimpunch_angle*2.);
        println!("angle: {:?}",game_data.local_player.aimpunch_angle);

        //let dist_from_head = glm::distance(&point.into(), &to.into());
        let entity = &game_data.entity_list.entities[closest_player];
        let vel = entity.vec_velocity *2.;
        let dist_from_head = get_dist_from_crosshair(
            entity.head_pos + vel,
            game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            angles
        );
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
        println!("dist from body: {}", dist_from_body);
        if dist_from_head < 5.
        || dist_from_neck < 6.
        || dist_from_body < 7.
        || dist_from_middle < 7.
        || dist_from_lower < 7.
        {
            //game_data.local_player.incross = closest_player as i32;
            port.write(b"ml\n").unwrap();
        }
        
    }

        // if game_data.local_player.incross > 0 && game_data.local_player.incross <= 64 {
        //     //info!("incross: {}", game_data.local_player.incross);
        //     if let Some(enemy_team) = game_data.entity_list.get_team_for((game_data.local_player.incross as usize) -1) {
        //         //println!("enemy team: {}", enemy_team);
        //         if enemy_team != game_data.local_player.team_num && game_data.local_player.aimpunch_angle > -0.04 {
        //             port.write(b"m0\n").unwrap();
        //             //print!("firing {}", game_data.local_player.aimpunch_angle);
        //         }
        //     }
        // }
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