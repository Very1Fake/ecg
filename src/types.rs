use glam::f32::{Mat4, Quat, Vec3, Vec4};
use winit::event_loop::EventLoop as WEventLoop;

// Low-level types

pub type EventLoop = WEventLoop<()>;

// Graphics related types

pub type Float32x3 = Vec3;
pub type Float32x4 = Vec4;

pub type Matrix4 = Mat4;
pub type RawMatrix4 = [[f32; 4]; 4];

pub type Rad = f32;
pub type Rotation = Quat;

// World related types

pub type Position = Vec3;
