use serialport::SerialPort;
use crate::{gamedata::GameData, datatypes:: game::WeaponId};


/// returns true if you should continue after and false if you should return from func early
pub fn zues_knife_bot(port: &mut Box<dyn SerialPort>, game_data: &GameData, closest_player: usize) -> bool {
    let entity = &game_data.entity_list.entities[closest_player];

    // zuesbot
    let entity_world_distance = 
            (
                entity.head_pos
                - (game_data.local_player.vec_origin + game_data.local_player.vec_view_offset)
            ).magnitude();
    if game_data.local_player.weapon_id == WeaponId::Taser {
        if entity_world_distance >= 182.5 {return false}
    }
    if game_data.local_player.weapon_id == WeaponId::Knife {
        println!("dist: {}", entity_world_distance);
        if entity_world_distance >= 70. {return false}
        if entity_world_distance < 45. {
            port.write(b"mr\n").unwrap();
            return false;
        }
    }
    true
}