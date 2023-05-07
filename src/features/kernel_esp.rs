use memflow::prelude::{Pod, v1::*};
use memflow_win32::prelude::v1::*;
use ::std::{ops::Add, time::SystemTime, convert::TryInto};
use crate::{offsets::{find_pattern}, gamedata::GameData, utils::math, datatypes::{tmp_vec3, tmp_vec2}};


const BUFFER_MAX: usize = 32;

#[repr(C)]
#[derive(Pod)]
struct DXCOLOR {
    col: u32
}

impl DXCOLOR {
    fn from_rgb(r:u32,g:u32,b:u32)-> Self {
        let v: u32 = (r as u32) | (g as u32) << 8 | (b as u32) << 16;
        Self {col:v }
    }
}

#[repr(C)]
#[derive(Pod,Debug)]
struct BoxCommand {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[repr(C)]
#[derive(Pod)]
struct RelativePoint {
    x: i16,
    y: i16,
}

#[repr(C)]
#[derive(Pod)]
struct Bones {
    head: RelativePoint,
    body: RelativePoint,
    lhand: RelativePoint,
    rhand: RelativePoint,
    lfoot: RelativePoint,
    rfoot: RelativePoint,
}

#[repr(C)]
#[derive(Pod)]
struct BoxCommandBuffer {
    // we skip past the signature address of the struct (4 bytes)
    // signature: u32, // 0x0

    // sig2: u32, // 0x4
    // sig3: u32,// 0x08
    
    reading: i32,      // 0x0C
    thickness: u32, // 0x10

    color: u32,  // 0x14

    draw_count: u32,   // 0x18
    draw_ready: u32,   // 0x1C

    // then buffer happens here
    //buffer: [BoxCommandPosOnly;32]
}

pub struct KernEsp <T,V> {
    os: Win32Kernel<T,V>,
    mod_base: Address,
    buffer_addr: umem,
    //buffer_info: BoxCommandBuffer,
    last_name_update: SystemTime,
    screen_width: u32,
    screen_height: u32,
}

impl <T: 'static + PhysicalMemory + Clone, V: 'static + VirtualTranslate2 + Clone>
    KernEsp<T,V>
{
    pub fn new(mut os: Win32Kernel<T,V>, driver_location: Option<(Address,umem)>, screen_size: Option<(u32,u32)>) -> Result<Self> {//Win32Kernel<impl PhysicalMemory + Clone, impl VirtualTranslate2 + Clone>
        // we have a longer sig for the kernel version cause we gotta search the kernel space
        let buffer_magic = "0D F0 CC C0 C0 CC C0 CC F0 0D F0 0D";

        //os.module_by_name("DxDr.sys")?;
        
        let (mod_base, mod_size) = if let Some(scan_location) = driver_location {
            scan_location
        } else {
            let os_info = os.info();
            println!("Got OS Info {os_info:?}");
            (
                0x0.into(),
                os_info.base.to_umem()
            )
        };

        let dump = os.read_raw(mod_base, mod_size as usize).data_part()?;
        let addr = find_pattern(&dump, buffer_magic).ok_or(Error(ErrorOrigin::Memory, ErrorKind::NotFound)/*.log_error("Failed to find ESP Buffer signature.")*/)? + 0xC;// offset to skip past sig
        println!("*.* Found Kernel ESP Buffer Address: {addr:#02x}");

        //let buffer_info: BoxCommandBuffer = unsafe {std::mem::transmute_copy::<[u8;8],BoxCommandBuffer>(&dump[addr..addr+std::mem::size_of::<BoxCommandBuffer>()].try_into().unwrap())};
        //let buffer_info: BoxCommandBuffer = os.read(mod_base + addr).data()?;

        let (screen_width, screen_height) = screen_size.unwrap_or((1920,1080));
        Ok(
            Self {
                os,
                mod_base,
                buffer_addr: addr as umem,
                //buffer_info,
                last_name_update: SystemTime::now(),
                screen_width,
                screen_height
                
            }
        )
    }

    /// Takes in the player locations from game_data
    /// Then computes screen positions for each players box
    /// Finally, writes these boxes to our box array in our render DLL
    pub fn render_esp(&mut self, game_data: &GameData) {
        
        let mut boxes = vec![];
        let mut relative_bones = vec![];
        let mut idxbuf = vec![];

        for (i,e) in game_data.entity_list.entities.iter().enumerate() {
            if e.team_num == game_data.local_player.team_num {continue}
            if e.lifestate > 0 {continue}
            if e.dormant &1 == 1 {continue}
            if game_data.local_player.observing_id == 0 || i == game_data.local_player.observing_id as usize -1 {continue}
            //if i == game_data.local_player.ent_idx {continue}
            
            let head_w2s = math::world_2_screen(&(e.head_pos+tmp_vec3{x:0.,y:0.,z:5.}),&game_data.vm, Some(self.screen_width as f32), Some(self.screen_height as f32));
            let origin_w2s = math::world_2_screen(&e.vec_origin,&game_data.vm, Some(self.screen_width as f32), Some(self.screen_height as f32));
            let right_foot_w2s = math::world_2_screen(&e.right_foot_pos,&game_data.vm, Some(self.screen_width as f32), Some(self.screen_height as f32));//.unwrap_or_default();
            let left_foot_w2s = math::world_2_screen(&e.left_foot_pos,&game_data.vm, Some(self.screen_width as f32), Some(self.screen_height as f32));//.unwrap_or_default();
            let body_w2s = math::world_2_screen(&e.lower_body_pos, &game_data.vm, Some(self.screen_width as f32), Some(self.screen_height as f32));


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
                x: 0.0f32.max(esp_box.left).min(self.screen_width as f32),
                y: 0.0f32.max(esp_box.top).min(self.screen_height as f32),
            };
            //let bottom_right = origin_w2s.unwrap_or(head_w2s.unwrap_or_default() + tmp_vec3 { x:foot_dist, y:0.0, z:0.0 }) + tmp_vec3 {x: 5.0, y: 0.0, z: 0.0};
            let wh = tmp_vec2 {
                x: 0.0f32.max(esp_box.right).min(self.screen_width as f32) - top_left.x,
                y: 0.0f32.max(esp_box.bottom).min(self.screen_height as f32) - top_left.y,
            };

            boxes.push(BoxCommand {
                x: top_left.x as u32,
                y: top_left.y as u32,
                w: wh.x as u32,
                h: wh.y as u32,
            });

            let head = head_w2s.unwrap_or(tmp_vec3 {x:top_left.x,y:top_left.y,z:0.0});
            let body = body_w2s.unwrap_or(tmp_vec3 {x:top_left.x,y:top_left.y,z:0.0});
            let lfoot = left_foot_w2s.unwrap_or(tmp_vec3 {x:top_left.x,y:top_left.y,z:0.0});
            let rfoot = right_foot_w2s.unwrap_or(tmp_vec3 {x:top_left.x,y:top_left.y,z:0.0});
            let lhand = tmp_vec3 {x:top_left.x,y:top_left.y+50.0,z:0.0};
            let rhand = tmp_vec3 {x:top_left.x+100.0,y:top_left.y+50.0,z:0.0};

            relative_bones.push(Bones{
                head: RelativePoint { x: (head.x-top_left.x) as i16, y: (head.y-top_left.y) as i16 },
                body: RelativePoint { x: (body.x-top_left.x) as i16, y: (body.y-top_left.y) as i16 },
                lhand: RelativePoint { x: (lhand.x-top_left.x) as i16, y: (lhand.y-top_left.y) as i16 },
                rhand: RelativePoint { x: (rhand.x-top_left.x) as i16, y: (rhand.y-top_left.y) as i16 },
                lfoot: RelativePoint { x: (lfoot.x-top_left.x) as i16, y: (lfoot.y-top_left.y) as i16 },
                rfoot: RelativePoint { x: (rfoot.x-top_left.x) as i16, y: (rfoot.y-top_left.y) as i16 },
            });

            let idx = i as u16;
            let idxb = idx.to_le_bytes();
            idxbuf.push(idxb);


            //println!("pushed esp box for {i}");
        }

        let command_size =  std::mem::size_of::<BoxCommand>();
        let buffer_addr = self.buffer_addr + std::mem::size_of::<BoxCommandBuffer>() as umem;
        let bs;
        let finalbuf;
        let finalbonebuf;
        let finalidxbuf;
        let mut batch = self.os.batcher();

        // if let Ok(elap) = self.last_name_update.elapsed() {

        // }

        let mut boxbuf = vec![];
        let mut bonebuf = vec![];//relative_bones.as_bytes();

        //let mut idx;
        for (i,draw_box) in boxes.iter().enumerate() {
            //println!("writing name ptr {} for idx {}", draw_box.name_ptr, i);
            //batch.write_into(self.mod_base + (buffer_addr + (i as u64 * command_size as u64)), draw_box);
            if i >= BUFFER_MAX {break;} // break if we reach the max buffer of 32 (no overflow pls)
            //print!("drawing box: {draw_box:?}");
            boxbuf.push(draw_box.as_bytes());
            bonebuf.push(relative_bones[i].as_bytes());
        }

        finalbuf = boxbuf.concat();
        finalbonebuf = bonebuf.concat();
        finalidxbuf = idxbuf.concat();
        
        batch.write_raw_into(self.mod_base + buffer_addr, &finalbuf);
        batch.write_raw_into(self.mod_base + buffer_addr + command_size*BUFFER_MAX, &finalbonebuf);
        batch.write_raw_into(self.mod_base + buffer_addr + command_size*BUFFER_MAX + std::mem::size_of::<Bones>() * BUFFER_MAX, &finalidxbuf);

        // set the draw_ready bitand count
        //proc.write((self.buffer_addr + 0x4).into(), &1).data().unwrap();

        let size = boxes.len() as u32;
        bs = [DXCOLOR::from_rgb(100, 100, 160).as_bytes(),size.as_bytes(),1u32.as_bytes()].concat();
        //println!("writing {:#02x?}", bs);
        
        batch.write_raw_into(self.mod_base + (self.buffer_addr+ 0x8), &bs);
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