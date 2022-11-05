use log::info;

use crate::{human_interface::HumanInterface, gamedata::GameData, datatypes::tmp_vec2};
use super::RecoilRecorder;


pub fn recoil_replay(game_data: &GameData, recoil_data: &RecoilRecorder, human: &mut HumanInterface) {
    let shots = game_data.local_player.shots_fired;
    if shots < 2 {return}
    // recoil data starts at shotsfired one but is indexed from 0
    // so current shot is actually one shot ahead which is what we want
    let next_recoil = recoil_data.get_recoil_at(shots as usize,game_data.local_player.weapon_id);

    if let Some(recoil_data) = next_recoil {
        if let Some(screen) = recoil_data.screen_pos {
            human.add_goal(screen);
        }
    }
    
}

// will recoil with live data instead of recorded
// pub fn recoil_live(game_data: &GameData, recoil_data: &RecoilRecorder, human: &mut HumanInterface) {
//     let new_angle = game_data.local_player.aimpunch_angle *2.;
//     let crosshair_world = math::get_crosshair_world_point_at_dist(
//         10.,
//         game_data.local_player.vec_origin + game_data.local_player.vec_view_offset,
//         angles
//     );
// }