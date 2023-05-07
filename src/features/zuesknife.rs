use crate::{gamedata::GameData, datatypes:: game::WeaponId, human_interface::HumanInterface};


/// returns true if you should continue after and false if you should return from func early
pub fn zues_knife_bot(human: &mut HumanInterface, game_data: &GameData, closest_player: usize) -> bool {
    let entity = &game_data.entity_list.entities[closest_player];

    // zuesbot
    let entity_world_distance = 
            (
                entity.head_pos
                - (game_data.entity_list.local_player.vec_origin + game_data.entity_list.local_player.vec_view_offset)
            ).magnitude();
    if game_data.entity_list.local_player.weapon_id == WeaponId::Taser {
        if entity_world_distance >= 182.5 {return false}
    }
    if game_data.entity_list.local_player.weapon_id == WeaponId::Knife {
        //println!("dist: {}", entity_world_distance);
        if entity_world_distance >= 70. {return false}
        if entity_world_distance < 45. {
            human.mouse_right().expect("failed to send mouse right click, serial must have disconnected");
            return false;
        }
    }
    true
}