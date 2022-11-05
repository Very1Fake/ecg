use std::time::Duration;

use winit::event::{ElementState, VirtualKeyCode};

use crate::types::{F32x2, F32x3, Matrix4, Rad};

/// Represents camera mode
#[derive(Debug)]
pub enum CameraMode {
    // TODO: FirstPerson
    ThirdPerson {
        // Distance between camera and target
        distance: f32,
    },
}

impl CameraMode {
    pub const DEFAULT_DISTANCE: f32 = 2.5;
}

impl Default for CameraMode {
    fn default() -> Self {
        Self::ThirdPerson {
            distance: Self::DEFAULT_DISTANCE,
        }
    }
}

/// Represents camera and its dependents state
#[derive(Debug)]
pub struct Camera {
    /// Eye position
    pub position: F32x3,
    /// Position of the target
    pub target: F32x3,

    pub yaw: f32,
    pub pitch: f32,

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
    pub const DEFAULT_POSITION: F32x3 = F32x3::new(0.0, 0.5, 5.0);
    pub const DEFAULT_TARGET: F32x3 = F32x3::ZERO;
    pub const DEFAULT_YAW: f32 = -90.0;
    pub const DEFAULT_PITCH: f32 = 15.0;
    pub const DEFAULT_FOV: f32 = 45.0;
    pub const Z_NEAR: f32 = 0.1;
    pub const Z_FAR: f32 = 100.0;

    // TODO: Split camera and player logic
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Self::DEFAULT_POSITION,
            target: Self::DEFAULT_TARGET,
            yaw: Self::DEFAULT_YAW.to_radians(),
            pitch: Self::DEFAULT_PITCH.to_radians(),
            aspect,
            mode: CameraMode::default(),
            fov: Self::DEFAULT_FOV,
            near: Self::Z_NEAR,
            far: Self::Z_FAR,
        }
    }

    /// Resize projection
    pub fn proj_resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// Calculate projection matrix
    ///
    /// Projection matrix warps the scene to give the effect of depth
    pub fn proj_mat(&self) -> Matrix4 {
        Matrix4::perspective_lh(self.fov, self.aspect, self.near, self.far)
    }

    /// Calculate camera view matrix
    ///
    /// Camera view matrix moves the world to be at the position and rotation of the camera
    pub fn view_mat(&self) -> Matrix4 {
        match self.mode {
            CameraMode::ThirdPerson { .. } => {
                Matrix4::look_at_lh(self.position, self.target, F32x3::Y)
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
}

#[derive(Debug)]
pub struct CameraController {
    pub forward: f32,
    pub backward: f32,
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub horizontal: f32,
    pub vertical: f32,
    pub zoom: f32,
    pub sensitivity: f32,
}

impl CameraController {
    const SPEED: f32 = 2.0;
    const MIN_DISTANCE: f32 = 0.5;
    const SAFE_PITCH: f32 = 1.57;

    /// Resets camera controller inputs
    pub fn reset(&mut self) {
        self.forward = 0.0;
        self.backward = 0.0;
        self.left = 0.0;
        self.right = 0.0;
        self.up = 0.0;
        self.down = 0.0;
        self.horizontal = 0.0;
        self.vertical = 0.0;
        self.zoom = 0.0;
    }

    /// Processes input from keyboard
    pub fn virtual_key(&mut self, key: VirtualKeyCode, state: ElementState) {
        let force = if matches!(state, ElementState::Pressed) {
            1.0
        } else {
            0.0
        };

        match key {
            // Move forward
            VirtualKeyCode::W | VirtualKeyCode::Up => self.forward = force,
            // Move left
            VirtualKeyCode::A | VirtualKeyCode::Left => self.left = force,
            // Move backward
            VirtualKeyCode::S | VirtualKeyCode::Down => self.backward = force,
            // Move right
            VirtualKeyCode::D | VirtualKeyCode::Right => self.right = force,
            // Move up
            VirtualKeyCode::Space => self.up = force,
            // Move down
            VirtualKeyCode::LShift => self.down = force,
            // Skip other keys
            _ => {}
        }
    }

    /// Processes input from mouse
    pub fn mouse_move(&mut self, delta: F32x2) {
        // Yaw angle
        self.horizontal = delta.x as f32;
        // Pitch angle
        self.vertical = delta.y as f32;
    }

    /// Processes input from mouse wheel
    pub fn mouse_wheel(&mut self, delta: f32) {
        self.zoom = delta;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, duration: Duration) {
        let duration = duration.as_secs_f32();
        let modifier = Self::SPEED * duration;

        match &mut camera.mode {
            CameraMode::ThirdPerson { distance } => {
                // Target forward/right vector
                let (forward, right, length) = {
                    let forward = camera.target - camera.position;
                    let right = forward.normalize().cross(F32x3::Y);

                    (F32x3::Y.cross(right), right, forward.length())
                };

                // Zoom in/out
                {
                    let new = -((self.zoom * length * 0.75) * modifier);

                    if *distance + new > Self::MIN_DISTANCE {
                        *distance += new;
                    } else {
                        *distance = Self::MIN_DISTANCE;
                    }
                }

                // Move forward/backward
                camera.target += forward * (self.forward - self.backward) * modifier;
                // Move left/right
                camera.target += right * (self.left - self.right) * modifier;
                // Move up/down
                camera.target.y += (self.up - self.down) * modifier;

                // Rotate camera
                camera.yaw += self.horizontal.to_radians() * self.sensitivity * modifier;
                camera.pitch += self.vertical.to_radians() * self.sensitivity * modifier;

                // Pitch angle safety
                if camera.pitch < -Self::SAFE_PITCH {
                    camera.pitch = -Self::SAFE_PITCH;
                } else if camera.pitch > Self::SAFE_PITCH {
                    camera.pitch = Self::SAFE_PITCH;
                }

                // Calculate camera position
                {
                    let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
                    let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();

                    let hor_dist = *distance * pitch_cos;
                    let vert_dist = *distance * pitch_sin;

                    let offset_x = hor_dist * yaw_sin;
                    let offset_z = hor_dist * yaw_cos;

                    camera.position.x = camera.target.x - offset_x;
                    camera.position.z = camera.target.z - offset_z;
                    camera.position.y = camera.target.y + vert_dist;
                }

                // Reset mouse inputs
                self.zoom = 0.0;
                self.horizontal = 0.0;
                self.vertical = 0.0;
            }
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            forward: 0.0,
            backward: 0.0,
            left: 0.0,
            right: 0.0,
            up: 0.0,
            down: 0.0,
            horizontal: 0.0,
            vertical: 0.0,
            zoom: 0.0,
            sensitivity: 150.0,
        }
    }
}
