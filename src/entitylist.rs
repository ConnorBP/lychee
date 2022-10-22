use ::std::ops::Add;

use memflow::prelude::v1::*;
use log::info;

#[repr(C)]
#[derive(Copy, Clone,Debug, Pod)]
pub struct tmp_vec2 {
    x: f32,
    y: f32,
}

#[repr(C)]
#[derive(Copy, Clone,Debug, Pod)]
pub struct tmp_vec3 {
    x: f32,
    y: f32,
    z: f32,
}


#[derive(Copy, Clone,Debug)]
#[repr(C)]
pub struct EntityInfo {
    b_dormant: u8,
    b_is_local_player: u8,
    health: i32,
    team_num: i32,
    b_is_enemy: u8,

    vec_origin: tmp_vec3,
    vec_view_offset: tmp_vec3,
    view_angles: tmp_vec3,

    vec_feet: tmp_vec2,
    vec_head: tmp_vec2,
}

// unsafe impl Pod for EntityInfo {
//     fn zeroed() -> Self where Self: Sized {
// 		unsafe { ::std::mem::zeroed() }
// 	}

//     fn as_bytes(&self) -> &[u8] {
// 		unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, ::std::mem::size_of_val(self)) }
// 	}

//     fn as_bytes_mut(&mut self) -> &mut [u8] {
// 		unsafe { core::slice::from_raw_parts_mut(self as *mut _ as *mut u8, ::std::mem::size_of_val(self)) }
// 	}

//     fn as_data_view(&self) -> &DataView {
// 		unsafe { ::std::mem::transmute(self.as_bytes()) }
// 	}

//     fn as_data_view_mut(&mut self) -> &mut DataView {
// 		unsafe { ::std::mem::transmute(self.as_bytes_mut()) }
// 	}

//     fn transmute<T: Pod>(self) -> T where Self: Sized {
// 		assert_eq!(mem::size_of::<Self>(), mem::size_of::<T>(), "Self must have equal size to target type");
// 		let result = unsafe { ::std::mem::transmute_copy(&self) };
// 		::std::mem::forget(self);
// 		result
// 	}

//     fn transmute_ref<T: Pod>(&self) -> &T where Self: Sized {
// 		assert_eq!(mem::size_of_val(self), mem::size_of::<T>(), "Self must have equal size to target type");
// 		assert!(mem::align_of_val(self) >= mem::align_of::<T>(), "Align of `Self` must be ge than `T`");
// 		unsafe { &*(self as *const Self as *const T) }
// 	}

//     fn transmute_mut<T: Pod>(&mut self) -> &mut T where Self: Sized {
// 		assert_eq!(mem::size_of_val(self), mem::size_of::<T>(), "Self must have equal size to target type");
// 		assert!(mem::align_of_val(self) >= mem::align_of::<T>(), "Align of `Self` must be ge than `T`");
// 		unsafe { &mut *(self as *mut Self as *mut T) }
// 	}

//     fn _static_assert() {}
// }

pub fn get_entity_addr(proc: &mut (impl Process + MemoryView), client_module_addr: Address, for_index: u32) -> Result<Address> {
    let entity = proc.read_addr32(client_module_addr.add(*crate::DW_ENTITYLIST + (for_index * 0x10))).data()?;
    info!("got entity: {:?} for index {}", entity, for_index);
    Ok(entity)
}

// pub fn yeet<P>(proc: &mut P) where P: Process + MemoryView {

// }