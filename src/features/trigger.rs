
use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::{gamedata::GameData, datatypes::game::WeaponId};

pub fn incross_trigger(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>, game_data: &GameData) {
    if !kb.is_down(0x06) {return}
    if game_data.local_player.shots_fired > 1 {return}
    //if game_data.local_player.vec_velocity.magnitude() > 0.1 {return}
        if game_data.local_player.incross > 0 && game_data.local_player.incross <= 64 {
            //info!("incross: {}", game_data.local_player.incross);
            let entity = &game_data.entity_list.entities[(game_data.local_player.incross as usize) -1];
            if let Some(enemy_team) = game_data.entity_list.get_team_for((game_data.local_player.incross as usize) -1) {
                if game_data.local_player.aimpunch_angle.magnitude() > 0.1 {return} // force acuracy
                // zuesbot
                if game_data.local_player.weapon_id == WeaponId::Taser {
                    let entity_world_distance = 
                        (
                            entity.head_pos
                            - (game_data.local_player.vec_origin + game_data.local_player.vec_view_offset)
                        ).magnitude();
                    if entity_world_distance >= 182.5 {return}
                }
                //info!("enemy team: {}", enemy_team);
                if enemy_team != game_data.local_player.team_num {
                    port.write(b"ml\n").unwrap();
                    //print!("firing {}", game_data.local_player.aimpunch_angle);
                }
            }
        }
}