use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use crate::{utils::math::get_dist_from_crosshair,gamedata::GameData, human_interface::HumanInterface};
use super::zuesknife;

// const PREFIRE_FACTOR: f64 = 5.;

#[derive(Default)]
pub struct AlgebraTrigger {
    // last_dist: f32,
    // speed_avg: f32,
}

impl AlgebraTrigger {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn algebra_trigger(&mut self, kb: &mut Win32Keyboard<impl MemoryView>, human: &mut HumanInterface, game_data: &GameData, delta: f64) {
        if !kb.is_down(0x06) {return}
        println!("Delta FPS: {}", 1./delta);
        if game_data.local_player.shots_fired > 1 {return}
        if game_data.local_player.aimpunch_angle.magnitude() > 0.05 {return} // force acuracy
        //info!("velocity: {} vec: {:?}", game_data.local_player.vec_velocity.magnitude(),game_data.local_player.vec_velocity);
        //if game_data.local_player.vec_velocity.magnitude() > 1. {return}
        if let Some(closest_player) = game_data.entity_list.closest_player {
            if game_data.entity_list.get_team_for(closest_player).unwrap_or(game_data.local_player.team_num) == game_data.local_player.team_num {return}
    
            let angles = game_data.local_player.view_angles + (game_data.local_player.aimpunch_angle*2.);
            //info!("angle: {:?}",game_data.local_player.aimpunch_angle);
            
            //let dist_from_head = glm::distance(&point.into(), &to.into());
            if !zuesknife::zues_knife_bot(human,game_data,closest_player) {return}

            let entity = &game_data.entity_list.entities[closest_player];

            //let vel = entity.vec_velocity.magnitude();
            //let norm = entity.vec_velocity.norm(vel);
            let dist_from_head = get_dist_from_crosshair(
                entity.head_pos,// + norm,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles.xy()
            );
    
            //let speed = (dist_from_head - self.last_dist) as f64 * delta;
            //println!("speed {}", self.speed_avg);
            //self.last_dist = dist_from_head;

            // speed factor. Bring speed value from something like 0.1-0.01 to more like 1.0-0.5 ish
            // value of speed is negative when moving towards enemy and positive when moving away
            //let sf = (speed * PREFIRE_FACTOR) as f32;
            //self.speed_avg = (self.speed_avg + sf) / 2.;
            //info!("dist from body: {}", dist_from_body);
            

            if dist_from_head < 5. {
                human.mouse_left().expect("failed to send mouse left click, serial must have disconnected");
                return;
            }
    
            let dist_from_neck = get_dist_from_crosshair(
                entity.neck_pos,// + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles.xy()
            );

            if dist_from_neck < 5. {
                human.mouse_left().expect("failed to send mouse left click, serial must have disconnected");
                return;
            }

            let dist_from_body = get_dist_from_crosshair(
                entity.upper_body_pos,// + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles.xy()
            );

            if dist_from_body < 7. {
                human.mouse_left().expect("failed to send mouse left click, serial must have disconnected");
                return;
            }

            // let dist_from_middle = get_dist_from_crosshair(
            //     entity.middle_body_pos,// + vel,
            //     game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
            //     angles
            // );

            let dist_from_lower = get_dist_from_crosshair(
                entity.lower_body_pos,// + vel,
                game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
                angles.xy()
            );

            if dist_from_lower < 7. {
                human.mouse_left().expect("failed to send mouse left click, serial must have disconnected");
                return;
            }
        }
    }
}