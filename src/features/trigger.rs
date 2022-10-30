
use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;
use serialport::SerialPort;
use crate::gamedata::GameData;

pub fn incross_trigger(kb: &mut Win32Keyboard<impl MemoryView>, port: &mut Box<dyn SerialPort>, game_data: &GameData) {
    if !kb.is_down(0x06) {return}
    if game_data.local_player.vec_velocity.magnitude() > 0.1 {return}
        if game_data.local_player.incross > 0 && game_data.local_player.incross <= 64 {
            //info!("incross: {}", game_data.local_player.incross);
            if let Some(enemy_team) = game_data.entity_list.get_team_for((game_data.local_player.incross as usize) -1) {
                //println!("enemy team: {}", enemy_team);
                if enemy_team != game_data.local_player.team_num && game_data.local_player.aimpunch_angle.x > -0.04 {
                    port.write(b"m0\n").unwrap();
                    //print!("firing {}", game_data.local_player.aimpunch_angle);
                }
            }
        }
}