// Low-level types

pub type EventLoop = winit::event_loop::EventLoop<()>;
pub type WEvent<'a> = winit::event::Event<'a, ()>;

pub type ProfileResult<'a> = (u8, &'a str, f64);

// Graphics related types

pub type U32x2 = glam::UVec2;

pub type F32x2 = glam::Vec2;
pub type F32x3 = glam::Vec3;
pub type F32x4 = glam::Vec4;

pub type Mat4 = glam::Mat4;
pub type RawMat4 = [[f32; 4]; 4];

pub type Rad = f32;
pub type Rotation = glam::Quat;

// World related types

pub type Position = glam::Vec3;
