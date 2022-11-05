use glam::{
    f32::{Mat4, Quat, Vec2, Vec3, Vec4},
    u32::UVec2,
};

// Low-level types

pub type EventLoop = winit::event_loop::EventLoop<()>;
pub type WEvent<'a> = winit::event::Event<'a, ()>;

// Graphics related types

pub type U32x2 = UVec2;

pub type F32x2 = Vec2;
pub type F32x3 = Vec3;
pub type F32x4 = Vec4;

pub type Matrix4 = Mat4;
pub type RawMatrix4 = [[f32; 4]; 4];

pub type Rad = f32;
pub type Rotation = Quat;

// World related types

pub type Position = Vec3;
