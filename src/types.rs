use glam::{
    f32::{Mat4, Vec3},
    Vec4,
};

// Graphics related types

pub type Float32x3 = Vec3;
pub type Float32x4 = Vec4;

pub type Matrix4 = Mat4;
pub type RawMatrix4 = [[f32; 4]; 4];

pub type Rad = f32;

// World related types

pub type Position = Vec3;
