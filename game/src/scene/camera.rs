use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_6, TAU},
    time::Duration,
};

use common::prof;
use winit::event::{ElementState, VirtualKeyCode};

use crate::types::{F32x2, F32x3, Matrix4, Rad};

/// Represents camera mode
#[derive(PartialEq, Debug)]
pub enum CameraMode {
    FirstPerson {
        /// Direction which camera is facing
        forward: F32x3,
    },
    ThirdPerson {
        /// Position of the target
        target: F32x3,
        /// Distance between camera and target
        distance: f32,
    },
}

impl CameraMode {
    pub const DEFAULT_FORWARD: F32x3 = F32x3::ONE;
    pub const DEFAULT_TARGET: F32x3 = F32x3::ZERO;
    pub const DEFAULT_DISTANCE: f32 = 2.5;

    pub const fn first_person() -> Self {
        Self::FirstPerson {
            forward: Self::DEFAULT_FORWARD,
        }
    }

    pub const fn third_person() -> Self {
        Self::ThirdPerson {
            target: Self::DEFAULT_TARGET,
            distance: Self::DEFAULT_DISTANCE,
        }
    }
}

/// Represents camera and its dependents state
#[derive(Debug)]
pub struct Camera {
    /// Eye position
    pub position: F32x3,

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
    pub const DEFAULT_POSITION: F32x3 = F32x3::new(5.0, 0.5, 0.0);
    pub const DEFAULT_YAW: f32 = -90.0;
    pub const DEFAULT_PITCH: f32 = 15.0;
    pub const DEFAULT_FOV: f32 = 45.0;
    pub const Z_NEAR: f32 = 0.1;
    pub const Z_FAR: f32 = 100.0;

    // TODO: Split camera and player logic
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Self::DEFAULT_POSITION,
            yaw: Self::DEFAULT_YAW.to_radians(),
            pitch: Self::DEFAULT_PITCH.to_radians(),
            aspect,
            mode: CameraMode::third_person(),
            fov: Self::DEFAULT_FOV.to_radians(),
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
            CameraMode::FirstPerson { forward } => {
                Matrix4::look_to_lh(self.position, forward, F32x3::Y)
            }
            CameraMode::ThirdPerson { target, .. } => {
                Matrix4::look_at_lh(self.position, target, F32x3::Y)
            }
        }
    }
}

#[derive(Debug)]
pub struct CameraController {
    forward: f32,
    backward: f32,
    left: f32,
    right: f32,
    up: f32,
    down: f32,
    horizontal: f32,
    vertical: f32,
    zoom: f32,
    sensitivity: f32,
}

impl CameraController {
    const SPEED: f32 = 2.0;
    const MIN_DISTANCE: f32 = 0.5;

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
        prof!(_guard, "Camera::update_camera");

        let duration = duration.as_secs_f32();
        let modifier = Self::SPEED * duration;

        // TODO: Use linear interpolation to smooth camera rotation
        // Apply camera rotation
        camera.yaw = (camera.yaw + self.horizontal.to_radians() * self.sensitivity * modifier)
            .rem_euclid(TAU);
        // Pitch angle safety
        camera.pitch = (camera.pitch + self.vertical.to_radians() * self.sensitivity * modifier)
            .min(FRAC_PI_2 - 0.001)
            .max(-FRAC_PI_2 + 0.001);

        // Common calculations
        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        let horizontal_forward = F32x3::new(yaw_sin, 0.0, yaw_cos).normalize();
        let horizontal_right = horizontal_forward.cross(F32x3::Y);

        // Move camera/target
        match &mut camera.mode {
            CameraMode::FirstPerson { forward } => {
                // Camera rotation
                *forward = F32x3::new(yaw_sin, -pitch_sin, yaw_cos);

                // Move up/down
                camera.position.y += (self.up - self.down) * modifier;
                // Move forward/backward
                camera.position += horizontal_forward * (self.forward - self.backward) * modifier;
                // Move left/right
                camera.position += horizontal_right * (self.left - self.right) * modifier;

                // Change camera FOV
                // Also clamp FOV between 30 and 135 degrees
                camera.fov = (camera.fov + -self.zoom * 0.5 * modifier).clamp(FRAC_PI_6, 2.356194);
            }
            CameraMode::ThirdPerson { target, distance } => {
                // Zoom in/out
                {
                    let new =
                        -((self.zoom * (*target - camera.position).length() * 0.75) * modifier);

                    if *distance + new > Self::MIN_DISTANCE {
                        *distance += new;
                    } else {
                        *distance = Self::MIN_DISTANCE;
                    }
                }

                // Move forward/backward
                *target += horizontal_forward * (self.forward - self.backward) * modifier;
                // Move left/right
                *target += horizontal_right * (self.left - self.right) * modifier;
                // Move up/down
                target.y += (self.up - self.down) * modifier;

                // Calculate camera position
                {
                    let hor_dist = *distance * pitch_cos;
                    let vert_dist = *distance * pitch_sin;

                    let offset_x = hor_dist * yaw_sin;
                    let offset_z = hor_dist * yaw_cos;

                    camera.position.x = target.x - offset_x;
                    camera.position.z = target.z - offset_z;
                    camera.position.y = target.y + vert_dist;
                }
            }
        }

        // Reset mouse inputs
        self.zoom = 0.0;
        self.horizontal = 0.0;
        self.vertical = 0.0;
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
