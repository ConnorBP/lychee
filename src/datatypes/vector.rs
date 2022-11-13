use std::ops::Div;
use ::std::{ops::{Add, Sub, Mul}};
use memflow::prelude::Pod;
use serde::{Serialize,Deserialize};

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
    /// Swizzling woohoo
    pub fn xy(&self) -> tmp_vec2 {
        tmp_vec2 {
            x: self.x,
            y: self.y,
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