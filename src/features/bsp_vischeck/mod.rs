// utilizes the VBSP crate (modified for csgo support)
// to check if two points intersect a bsp leaf

pub mod trace;

//use std::io::Read;
use vbsp::{Bsp, Vector, BrushFlags};

use crate::datatypes::tmp_vec3;

#[derive(Debug)]
pub struct VisibleCheck {
    current_map_name: String,
    current_map: Option<Bsp>,
}

impl VisibleCheck {

    pub fn new() -> Self {
        Self{
            current_map_name: "".to_string(),
            current_map: None,
        }
    }

    pub fn load_map(&mut self, map_name: String) {
        use std::fs::read;
        if let Ok(data) = read(format!("./assets/maps/{}.bsp", map_name)) {
            self.current_map = Bsp::read(&data).ok();
        }
    }

    pub fn is_visible(&self, start: tmp_vec3, end: tmp_vec3) -> (bool,tmp_vec3) {
       // println!("checking vis between {start:?} and {end:?}");

        

        if let Some(map) = &self.current_map {


            return trace::is_visible(map, &start, &end);


            // map.leaf(n)
            // there is a valid map bsp loaded, so do the vischeck calc

            // let leaf = map.leaf_at(map_coords(Vector{x:start.x,y:start.y,z:start.z}));
            // if (leaf.contents & (BrushFlags::SOLID.bits() as i32)) != 0 {
            //     println!("contents sold at {start:?}");
            //     return (false,start);
            // }




            // let mut dir = end - start;
            // let mut current_point = start;
            // let mut steps = dir.magnitude();

            // dir /= steps;

            // let mut bsp_leaf;

            // while steps > 0.0 {
            //     current_point += dir;
            //     bsp_leaf = map.leaf_at(map_coords(current_point));

            //     if (bsp_leaf.contents & (BrushFlags::SOLID.bits() as i32)) != 0 {
            //         //println!("ran into solid at {current_point:?}, {steps}");
            //         return (false,current_point);
            //     }
            //     steps = steps - 1.0;
            // }
            // return (true, tmp_vec3{x:0f32,y:0f32,z:0f32});



        }
        (false, tmp_vec3{x:0f32,y:0f32,z:0f32})
    }

}

impl Into<Vector> for tmp_vec3 {

    fn into(self) -> Vector {
        Vector {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

// pub fn map_coords<C: Into<Vector>>(vec: C) -> Vector {
//     let vec = vec.into();
//     Vector {
//         x: vec.y * UNIT_SCALE,
//         y: vec.z * UNIT_SCALE,
//         z: vec.x * UNIT_SCALE,
//     }
// }

pub fn map_coords<C: Into<Vector>>(vec: C) -> Vector {
    let vec = vec.into();
    Vector {
        x: vec.x,// * UNIT_SCALE,
        y: vec.y,// * UNIT_SCALE,
        z: vec.z,// * UNIT_SCALE,
    }
}

// 1 hammer unit is ~1.905cm
pub const UNIT_SCALE: f32 = 1.0 / (1.905 * 100.0);