use memflow::prelude::{Pod, v1::*};
use memflow_win32::prelude::v1::*;
use ::std::{ops::Add, time::SystemTime, convert::TryInto};
use crate::{offsets::{find_pattern}, gamedata::GameData, utils::math, datatypes::{tmp_vec3, tmp_vec2}};


const BUFFER_MAX: usize = 256;

#[repr(C)]
#[derive(Pod)]
struct DXCOLOR {
    col: u32
}

#[repr(C)]
#[derive(Pod)]
struct BoxCommand {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    //col: DXCOLOR,
}

#[repr(C)]
#[derive(Pod)]
struct BoxCommandBuffer {
    // we skip past the signature address of the struct (4 bytes)
    // signature: u32, // 0x0

    screen_width: u32, // 0x0
    screen_height: u32,// 0x04
    
    reading: i32,      // 0x08

    wants_unload: u32, // 0x0C
    safe_unload: u32,  // 0x10

    draw_count: u32,   // 0x14
    draw_ready: u32,   // 0x18

    // then buffer happens here
    //buffer: [BoxCommand;256]
}

pub struct Esp {
    mod_base: Address,
    buffer_addr: umem,
    buffer_info: BoxCommandBuffer,
}

impl Esp {
    pub fn new(proc: &mut (impl Process + MemoryView), module_location: Option<(Address,u32)>) -> Result<Self> {
        let buffer_magic = "0D F0 CC C0";
        
        let (mod_base, mod_size) = if let Some(modi) = module_location {
            modi
        } else {
            let esp_module = proc.module_by_name("GameRecord.dll")?;
            (
                esp_module.base,
                esp_module.size as u32
            )
        };
        let dump = proc.read_raw(mod_base, mod_size as usize).data_part()?;
        let addr = find_pattern(&dump, buffer_magic).ok_or(Error(ErrorOrigin::Memory, ErrorKind::NotFound)/*.log_error("Failed to find ESP Buffer signature.")*/)? + 0x4;// offset to skip past sig
        println!("*.* Found ESP Buffer Address: {addr:#02x}");

        //let buffer_info: BoxCommandBuffer = unsafe {std::mem::transmute_copy::<[u8;8],BoxCommandBuffer>(&dump[addr..addr+std::mem::size_of::<BoxCommandBuffer>()].try_into().unwrap())};
        let buffer_info: BoxCommandBuffer = proc.read(mod_base + addr).data()?;

        Ok(
            Self {
                mod_base,
                buffer_addr: addr as umem,
                buffer_info,
            }
        )
    }

    /// Takes in the player locations from game_data
    /// Then computes screen positions for each players box
    /// Finally, writes these boxes to our box array in our render DLL
    pub fn render_esp(&self, proc: &mut (impl Process + MemoryView), game_data: &GameData) {
        
        let mut boxes = vec![];
        
        for (i,e) in game_data.entity_list.entities.iter().enumerate() {
            if e.team_num == game_data.local_player.team_num {continue}
            if e.lifestate > 0 {continue}
            if e.dormant &1 == 1 {continue}
            //if i == game_data.local_player.ent_idx {continue}
            
            let head_w2s = math::world_2_screen(&(e.head_pos+tmp_vec3{x:0.,y:0.,z:5.}),&game_data.vm, Some(self.buffer_info.screen_width as f32), Some(self.buffer_info.screen_height as f32));
            let origin_w2s = math::world_2_screen(&e.vec_origin,&game_data.vm, Some(self.buffer_info.screen_width as f32), Some(self.buffer_info.screen_height as f32));
            let right_foot_w2s = math::world_2_screen(&e.right_foot_pos,&game_data.vm, Some(self.buffer_info.screen_width as f32), Some(self.buffer_info.screen_height as f32));//.unwrap_or_default();
            let left_foot_w2s = math::world_2_screen(&e.left_foot_pos,&game_data.vm, Some(self.buffer_info.screen_width as f32), Some(self.buffer_info.screen_height as f32));//.unwrap_or_default();
            let body_w2s = math::world_2_screen(&e.lower_body_pos, &game_data.vm, Some(self.buffer_info.screen_width as f32), Some(self.buffer_info.screen_height as f32));


            let mut screen_vec = vec![];

            if let Some(head) = head_w2s {
                screen_vec.push(head.xy());
            }
            if let Some(origin) = origin_w2s {
                screen_vec.push(origin.xy());
            }
            if let Some(right_foot) = right_foot_w2s {
                screen_vec.push(right_foot.xy());
            }
            if let Some(left_foot) = left_foot_w2s {
                screen_vec.push(left_foot.xy());
            }
            if let Some(body) = body_w2s {
                screen_vec.push(body.xy());
            }

            if screen_vec.len() == 0 {continue}

            let esp_box = find_edges(screen_vec);
            if esp_box.is_none() {continue}
            let esp_box = esp_box.unwrap();




            // let foot_dist = 
            // (foot_w2s.xy()
            // - origin_w2s
            // .unwrap_or(
            //     head_w2s.unwrap_or(
            //         tmp_vec3 { x:5.0, y:5.0, z:0.0 }
            //     )
            // )).magnitude();

            // janky way of making boxes
            //let top_left = head_w2s.unwrap_or(origin_w2s.unwrap_or_default() - tmp_vec3 { x:foot_dist, y:0.0, z:0.0 }) - tmp_vec3 {x: 5.0, y: 0.0, z: 0.0};
            let top_left = tmp_vec2 {
                x: 0.0f32.max(esp_box.left).min(self.buffer_info.screen_width as f32),
                y: 0.0f32.max(esp_box.top).min(self.buffer_info.screen_height as f32),
            };
            //let bottom_right = origin_w2s.unwrap_or(head_w2s.unwrap_or_default() + tmp_vec3 { x:foot_dist, y:0.0, z:0.0 }) + tmp_vec3 {x: 5.0, y: 0.0, z: 0.0};
            let wh = tmp_vec2 {
                x: 0.0f32.max(esp_box.right).min(self.buffer_info.screen_width as f32) - top_left.x,
                y: 0.0f32.max(esp_box.bottom).min(self.buffer_info.screen_height as f32) - top_left.y,
            };

            boxes.push(BoxCommand {
                x: top_left.x as i32,
                y: top_left.y as i32,
                w: wh.x as i32,
                h: wh.y as i32,
                //col: DXCOLOR{ col: 0},
            });
            //println!("pushed esp box for {i}");
        }

        let command_size =  std::mem::size_of::<BoxCommand>();
        let buffer_addr = self.buffer_addr + std::mem::size_of::<BoxCommandBuffer>() as umem;
        let bs;
        let mut batch = proc.batcher();
        for (i,draw_box) in boxes.iter().enumerate() {
            batch.write_into(self.mod_base + (buffer_addr + (i as u64 * command_size as u64)), draw_box);
        }

        // set the draw_ready bitand count
        //proc.write((self.buffer_addr + 0x4).into(), &1).data().unwrap();

        let size = boxes.len() as u32;
        bs = [size.as_bytes(),1u32.as_bytes()].concat();
        //println!("writing {:#02x?}", bs);
        
        batch.write_raw_into(self.mod_base + (self.buffer_addr+ 0x14), &bs);
        // batch.write_into(self.mod_base + (self.buffer_addr + 0x18), &1u32);
        // batch.write_into(self.mod_base + (self.buffer_addr + 0x14), &size);
        batch.commit_rw().data_part().unwrap();
        std::mem::drop(batch);
        

    }

}

pub struct BoxEdges {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
}


/// Given a list of screen cordinates finds the outer edges on each axis (x,y)
pub fn find_edges(coordinates: Vec<tmp_vec2>) -> Option<BoxEdges> {
    let mut top = None;
    let mut bottom = None;
    let mut left = None;
    let mut right = None;


    for pos in coordinates.iter() {
        // coords are from top left down and right

        // check for largest and smallest of each coord direction

        if top.is_none() || top.unwrap() > pos.y {
            top = Some(pos.y);
        }

        if bottom.is_none() || bottom.unwrap() < pos.y {
            bottom = Some(pos.y);
        }

        if left.is_none() || left.unwrap() > pos.x {
            left = Some(pos.x);
        }

        if right.is_none() || right.unwrap() < pos.x {
            right = Some(pos.x);
        }
    }

    if top.is_none() || bottom.is_none() || left.is_none() || right.is_none() {
        return None;
    }

    Some(BoxEdges {
        top: top.unwrap(),
        bottom: bottom.unwrap(),
        left: left.unwrap(),
        right: right.unwrap(),
    })
}