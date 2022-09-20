use std::time::Duration;

use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
};
use winit::event::{ElementState, VirtualKeyCode};

use crate::types::{Float32x3, Matrix4, Rad, RawMatrix4};

/// Helper struct to send camera data to shader
#[derive(Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct CameraUniform {
    projection: RawMatrix4,
}

impl CameraUniform {
    pub fn new(projection: RawMatrix4) -> Self {
        Self { projection }
    }
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            projection: Matrix4::IDENTITY.to_cols_array_2d(),
        }
    }
}

/// Stores all necessary binding for camera
/// Also updates camera buffer
pub struct CameraBind {
    pub layout: BindGroupLayout,
    pub buffer: Buffer,
    pub bind_group: BindGroup,
}

impl CameraBind {
    pub const LAYOUT_DESCRIPTOR: BindGroupLayoutDescriptor<'static> = BindGroupLayoutDescriptor {
        label: Some("Camera Bind Group Layout"),
        entries: &[BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

    fn camera_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&Self::LAYOUT_DESCRIPTOR)
    }

    fn camera_buffer(device: &Device, projection: CameraUniform) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: cast_slice(&[projection]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
    }

    fn camera_bind_group(device: &Device, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    pub fn new(device: &Device, camera: &Camera) -> Self {
        let layout = Self::camera_layout(device);
        let buffer = Self::camera_buffer(device, camera.uniform());
        let bind_group = Self::camera_bind_group(device, &layout, &buffer);

        Self {
            layout,
            buffer,
            bind_group,
        }
    }

    pub fn update_buffer(&self, queue: &Queue, uniform: &CameraUniform) {
        queue.write_buffer(&self.buffer, 0, cast_slice(&[*uniform]))
    }
}

/// Represents camera mode
#[derive(Debug)]
pub enum CameraMode {
    ThirdPerson,
    // TODO: ThirdPerson
}

/// Represents camera and its dependents state
#[derive(Debug)]
pub struct Camera {
    /// Eye position
    pub position: Float32x3,
    /// Position of the target
    pub target: Float32x3,

    /// Camera mode
    pub mode: CameraMode,

    /// Projection aspect ratio
    pub aspect: f32,
    /// Field of View degree
    pub fov: Rad,
    /// Near Z axis plane
    pub near: f32,
    /// Far Z axis plane
    pub far: f32,
}

impl Camera {
    pub fn new(
        position: Float32x3,
        target: Float32x3,
        width: u32,
        height: u32,
        fov: Rad,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            target,
            aspect: height as f32 / width as f32,
            mode: CameraMode::ThirdPerson,
            fov,
            near,
            far,
        }
    }

    /// Resize projection
    pub fn proj_resize(&mut self, width: u32, height: u32) {
        self.aspect = height as f32 / width as f32;
    }

    /// Calculate projection matrix
    ///
    /// Projection matrix warps the scene to give the effect of depth
    pub fn proj_mat(&self) -> Matrix4 {
        Matrix4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    /// Calculate camera view matrix
    ///
    /// Camera view matrix moves the world to be at the position and rotation of the camera
    pub fn camera_mat(&self) -> Matrix4 {
        match self.mode {
            CameraMode::ThirdPerson => {
                Matrix4::look_at_rh(self.position, self.target, Float32x3::Y)
            }
        }

        // First person camera matrix
        // Matrix4::look_at_rh(
        //     self.position,
        //     Float32x3::new(self.yaw.sin(), self.pitch.sin(), self.yaw.sin()).normalize(),
        //     Float32x3::Y,
        // )

        // Copy of private `Mat4::look_at_lh`
        // let f = Float32x3::new(self.yaw.sin(), self.pitch.sin(), self.yaw.sin()).normalize();
        // let s = Float32x3::Y.cross(f).normalize();
        // let u = f.cross(s);
        // Matrix4::from_cols(
        //     Float32x4::new(s.x, u.x, f.x, 0.0),
        //     Float32x4::new(s.y, u.y, f.y, 0.0),
        //     Float32x4::new(s.z, u.z, f.z, 0.0),
        //     Float32x4::new(-s.dot(self.position), -u.dot(self.position), -f.dot(self.position), 1.0),
        // )
    }

    /// Create camera uniform matrix to send to shader
    pub fn uniform(&self) -> CameraUniform {
        CameraUniform::new((self.proj_mat() * self.camera_mat()).to_cols_array_2d())
    }
}

#[derive(Default, Debug)]
pub struct CameraController {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

impl CameraController {
    pub const SPEED: f32 = 2.0;

    pub fn update(&mut self, key: VirtualKeyCode, state: ElementState) {
        let is_pressed = matches!(state, ElementState::Pressed);

        match key {
            // Move forward
            VirtualKeyCode::W | VirtualKeyCode::Up => self.forward = is_pressed,
            // Move left
            VirtualKeyCode::A | VirtualKeyCode::Left => self.left = is_pressed,
            // Move backward
            VirtualKeyCode::S | VirtualKeyCode::Down => self.backward = is_pressed,
            // Move right
            VirtualKeyCode::D | VirtualKeyCode::Right => self.right = is_pressed,
            // Skip other keys
            _ => {}
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, duration: Duration) {
        let duration = duration.as_secs_f32();
        let modifier = Self::SPEED * duration;

        // For first person camera
        // let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        // let forward = Float32x3::new(yaw_cos, 0.0, yaw_sin).normalize();
        // let right = Float32x3::new(-yaw_sin, 0.0, yaw_cos).normalize();

        match camera.mode {
            CameraMode::ThirdPerson => {
                let forward = camera.target - camera.position;
                let forward_norm = forward.normalize();
                let forward_mag = forward.length();

                // Move forward/backward
                if self.forward && forward_mag > modifier {
                    camera.position += forward_norm * modifier;
                }
                if self.backward {
                    camera.position -= forward_norm * modifier;
                }

                let right = forward_norm.cross(Float32x3::Y);

                let forward = camera.target - camera.position;
                let forward_mag = forward.length();

                // Move left/right
                if self.right {
                    camera.position =
                        camera.target - (forward + right * modifier).normalize() * forward_mag;
                }
                if self.left {
                    camera.position =
                        camera.target - (forward - right * modifier).normalize() * forward_mag;
                }
            }
        }
    }
}
