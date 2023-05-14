use std::ops::{Div, DivAssign, AddAssign, IndexMut};
use ::std::{ops::{Add, Sub, Mul, Index}};
use memflow::prelude::Pod;
use serde::{Serialize,Deserialize};
use vbsp::Vector;

#[repr(C)]
#[derive(Copy, Clone,Debug, Default, Pod, Serialize, Deserialize)]
pub struct tmp_vec2 {
    pub x: f32,
    pub y: f32,
}

impl tmp_vec2 {
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.x.powf(2.)+self.y.powf(2.))
    }
    pub fn norm(&self, magnitude: f32) -> Self {
        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
        }
    }
}

impl Add for tmp_vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}
impl Add<tmp_vec3> for tmp_vec2 {
    type Output = tmp_vec3;

    fn add(self, rhs: tmp_vec3) -> Self::Output {
        tmp_vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: rhs.z
        }
    }
}

impl Sub for tmp_vec2 {
    type Output = Self;
    fn sub(self,rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl Sub<tmp_vec3> for tmp_vec2 {
    type Output = tmp_vec3;

    fn sub(self, rhs: tmp_vec3) -> Self::Output {
        tmp_vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: rhs.z
        }
    }
}

impl Mul<f32> for tmp_vec2 {
    type Output = Self;
    fn mul(self,rhs:f32) -> Self::Output {
        Self{
            x: self.x*rhs,
            y: self.y*rhs
        }
    }
}

impl Div<f32> for tmp_vec2 {
    type Output = Self;
    fn div(self,rhs:f32) -> Self::Output {
        Self{
            x: self.x/rhs,
            y: self.y/rhs
        }
    }
}

impl From<cgmath::Vector2<f32>> for tmp_vec2 {
    fn from(other: cgmath::Vector2<f32>) -> Self {
        Self { x: other.x, y: other.y }
    }
}

impl From<cgmath::Vector3<f32>> for tmp_vec3 {
    fn from(other: cgmath::Vector3<f32>) -> Self {
        Self { x: other.x, y: other.y, z: other.z }
    }
}

#[repr(C)]
#[derive(Copy, Clone,Debug, Default, Pod, Serialize, Deserialize)]
pub struct tmp_vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[allow(dead_code)]
impl tmp_vec3 {
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.x.powf(2.)+self.y.powf(2.)+self.z.powf(2.))
    }
    pub fn norm(&self, magnitude: f32) -> Self {
        Self {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }
    pub fn dot(&self,other:tmp_vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    /// Swizzling woohoo
    pub fn xy(&self) -> tmp_vec2 {
        tmp_vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl Into<tmp_vec3> for Vector {
    fn into(self) -> tmp_vec3 {
        tmp_vec3{
            x: self.x,
            y: self.y,
            z: self.z
        }
    }
}

impl Add for tmp_vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}
impl Add<tmp_vec2> for tmp_vec3 {
    type Output = Self;

    fn add(self, rhs: tmp_vec2) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z
        }
    }
}

impl Sub for tmp_vec3 {
    type Output = Self;
    fn sub(self,rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl Sub<tmp_vec2> for tmp_vec3 {
    type Output = Self;

    fn sub(self, rhs: tmp_vec2) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z
        }
    }
}

impl Mul<f32> for tmp_vec3 {
    type Output = Self;
    fn mul(self,rhs:f32) -> Self::Output {
        Self{
            x: self.x*rhs,
            y: self.y*rhs,
            z: self.z*rhs
        }
    }
}

impl DivAssign<f32> for tmp_vec3 {

    fn div_assign(&mut self, rhs: f32) {
        self.x /=rhs;
        self.y /=rhs;
        self.z /=rhs;
    }
}

impl AddAssign<f32> for tmp_vec3 {

    fn add_assign(&mut self, rhs: f32) {
        self.x +=rhs;
        self.y +=rhs;
        self.z +=rhs;
    }
}

impl AddAssign<tmp_vec3> for tmp_vec3 {

    fn add_assign(&mut self, rhs: tmp_vec3) {
        self.x +=rhs.x;
        self.y +=rhs.y;
        self.z +=rhs.z;
    }
}

impl Index<usize> for tmp_vec3 {

    // Required method
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => &self.z,
        }
    }

    type Output = f32;
}

impl IndexMut<usize> for tmp_vec3 {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => &mut self.z,
        }
    }
}

impl Into<cgmath::Vector2<f32>> for tmp_vec2 {
    fn into(self) -> cgmath::Vector2<f32> {
        cgmath::vec2(self.x, self.y)
    }
}

impl Into<cgmath::Vector3<f32>> for tmp_vec3 {
    fn into(self) -> cgmath::Vector3<f32> {
        cgmath::vec3(self.x,self.y,self.z)
    }
}

impl From<(f32, f32, f32)> for tmp_vec3 {
    fn from((x,y,z): (f32, f32, f32)) -> Self {
        tmp_vec3 { x: x, y: y, z: z }
    }
}

impl From<(f32, f32)> for tmp_vec2 {
    fn from((x,y): (f32, f32)) -> Self {
        tmp_vec2 { x: x, y: y }
    }
}