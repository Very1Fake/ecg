use std::{
    f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU},
    time::Duration,
};

use common::prof;
use winit::event::{ElementState, VirtualKeyCode};

use crate::types::{F32x2, F32x3, Mat4, Rad};

/// Represents camera mode
#[derive(PartialEq, Eq, Debug)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson,
}

/// Represents camera and its dependents state
#[derive(Debug)]
pub struct Camera {
    /// Eye position
    pub pos: F32x3,
    /// Camera rotation (yaw & pitch)
    pub rot: F32x2,

    /// Camera mode
    pub mode: CameraMode,
    /// Distance between camera and player
    pub dist: f32,

    /// Projection aspect ratio
    pub aspect: f32,
    /// Field Of View
    pub fov: Rad,
    /// Near Z axis plane
    pub near: f32,
    /// Far Z axis plane
    pub far: f32,

    // Camera smoothness
    /// Desired position
    pub f_pos: F32x3,
    /// Desired camera rotation
    pub f_rot: F32x2,
    /// Desired distance between camera and player
    pub f_dist: f32,
    /// Desired Field Of View
    pub f_fov: Rad,

    // Settings
    /// Interpolate camera position
    pub smooth_position: bool,
    /// Interpolate camera rotation
    pub smooth_rotation: bool,
}

impl Camera {
    // Utils
    const POS_LERP_TIME: f32 = 0.1;
    const ROT_LERP_TIME: f32 = 20.0;
    const ROTATION_SCALE: f32 = 0.005;
    const SWITCH_DISTANCE: f32 = 0.5;

    // Limits
    pub const MIN_DISTANCE: f32 = 0.1;
    pub const MIN_THIRD_PERSON_DISTANCE: f32 = 2.5;
    pub const MIN_FOV: f32 = FRAC_PI_4;
    pub const MAX_FOV: f32 = 2.356194;
    pub const MIN_Z_NEAR: f32 = 0.01;
    pub const MAX_Z_NEAR: f32 = 16.0;
    pub const MIN_Z_FAR: f32 = 32.0;
    pub const MAX_Z_FAR: f32 = 16384.0;

    // Defaults
    pub const DEFAULT_POSITION: F32x3 = F32x3::new(5.0, 0.5, 0.0);
    pub const DEFAULT_ORIENTATION: F32x2 = F32x2::new(-FRAC_PI_2, 0.08333);
    pub const DEFAULT_DISTANCE: f32 = 2.5;
    pub const DEFAULT_FOV: f32 = 90.0;
    pub const Z_NEAR: f32 = 0.1;
    pub const Z_FAR: f32 = 512.0;

    // TODO: Split camera and player logic
    pub fn new(aspect: f32, mode: CameraMode) -> Self {
        let dist = match mode {
            CameraMode::FirstPerson => Self::MIN_DISTANCE,
            CameraMode::ThirdPerson => Self::DEFAULT_DISTANCE,
        };

        Self {
            pos: Self::DEFAULT_POSITION,
            rot: Self::DEFAULT_ORIENTATION,
            aspect,
            mode: CameraMode::FirstPerson,
            dist,
            fov: Self::DEFAULT_FOV.to_radians(),
            near: Self::Z_NEAR,
            far: Self::Z_FAR,
            f_pos: Self::DEFAULT_POSITION,
            f_rot: Self::DEFAULT_ORIENTATION,
            f_dist: dist,
            f_fov: Self::DEFAULT_FOV.to_radians(),
            smooth_position: true,
            smooth_rotation: false,
        }
    }

    /// Resize projection
    pub fn proj_resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    /// Calculate projection matrix
    ///
    /// Projection matrix warps the scene to give the effect of depth
    pub fn proj_mat(&self) -> Mat4 {
        Mat4::perspective_lh(self.fov, self.aspect, self.near, self.far)
    }

    /// Calculate camera view matrix
    ///
    /// Camera view matrix moves the world to be at the position and rotation of the camera
    pub fn view_mat(&self) -> Mat4 {
        Mat4::from_translation(F32x3::new(0.0, 0.0, self.dist))
            * Mat4::from_rotation_x(-self.rot.y)
            * Mat4::from_rotation_y(-self.rot.x)
            * Mat4::from_translation(-self.pos)
    }

    /// Rotate camera
    pub fn rotate(&mut self, delta: F32x2) {
        self.f_rot = clamp(self.f_rot + delta * Self::ROTATION_SCALE);
    }

    /// Handle zoom
    pub fn zoom(&mut self, delta: f32) {
        if delta > 0.0 || !matches!(self.mode, CameraMode::FirstPerson { .. }) {
            let f_dist = self.dist + delta;
            match self.mode {
                CameraMode::FirstPerson { .. } => {
                    self.set_mode(CameraMode::ThirdPerson);
                    self.f_dist = Self::MIN_THIRD_PERSON_DISTANCE;
                }
                CameraMode::ThirdPerson { .. } => {
                    if f_dist < Self::SWITCH_DISTANCE {
                        self.set_mode(CameraMode::FirstPerson)
                    } else {
                        self.f_dist = f_dist;
                    }
                }
            }
        }
    }

    /// Set camera mode
    pub fn set_mode(&mut self, mode: CameraMode) {
        match mode {
            CameraMode::FirstPerson { .. } => {
                self.mode = mode;
                self.f_dist = Self::MIN_DISTANCE;
            }
            CameraMode::ThirdPerson { .. } => {
                self.mode = mode;
                self.f_dist = Self::DEFAULT_DISTANCE;
            }
        }
    }

    /// Update camera (basically dfo interpolation)
    pub fn update(&mut self, duration: Duration) {
        prof!(_guard, "Camera::update_camera");

        let dur = duration.as_secs_f32();

        // Interpolate camera distance
        if (self.dist - self.f_dist).abs() > 0.01 {
            self.dist = lerp(self.dist, self.f_dist, 0.75 * dur / Self::POS_LERP_TIME)
        }

        // Interpolate camera distance
        if (self.fov - self.f_fov).abs() > 0.01 {
            self.fov = lerp(self.fov, self.f_fov, 0.75 * dur / Self::POS_LERP_TIME)
        }

        // Interpolate camera position
        if self.smooth_position {
            if (self.pos - self.f_pos).length_squared() > 0.0001 {
                self.pos = self.pos.lerp(self.f_pos, 0.75 * dur / Self::POS_LERP_TIME)
            }
        } else {
            self.pos = self.f_pos;
        }

        // Interpolate camera rotation
        self.rot = if self.smooth_rotation {
            clamp(F32x2::new(
                lerp_angle(self.rot.x, self.f_rot.x, Self::ROT_LERP_TIME * dur),
                lerp(self.rot.y, self.f_rot.y, Self::ROT_LERP_TIME * dur),
            ))
        } else {
            self.f_rot
        };
    }

    /// Get camera forward unit vector on the XY plane
    pub fn forward_xy(&self) -> F32x3 {
        let (yaw_sin, yaw_cos) = self.rot.x.sin_cos();
        F32x3::new(yaw_sin, 0.0, yaw_cos)
    }
}

fn lerp(lhs: f32, rhs: f32, f: f32) -> f32 {
    // More precise, less performant
    lhs * (1.0 - f) + (rhs * f)
    // Less precise, more performant
    // lhs + f * (rhs - lhs)
}

fn lerp_angle(lhs: f32, rhs: f32, f: f32) -> f32 {
    lhs + f * {
        let t = (rhs - lhs).rem_euclid(TAU);
        (2.0 * t).rem_euclid(TAU) - t
    }
}

fn clamp(rot: F32x2) -> F32x2 {
    F32x2::new(
        rot.x.rem_euclid(TAU),
        rot.y.min(FRAC_PI_2 - 0.001).max(-FRAC_PI_2 + 0.001),
    )
}

#[derive(Debug)]
pub struct CameraController {
    forward: f32,
    backward: f32,
    left: f32,
    right: f32,
    up: f32,
    down: f32,
}

impl CameraController {
    const SPEED: f32 = 25.0;

    /// Resets camera controller inputs
    pub fn reset(&mut self) {
        self.forward = 0.0;
        self.backward = 0.0;
        self.left = 0.0;
        self.right = 0.0;
        self.up = 0.0;
        self.down = 0.0;
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

    // TODO: Put in players logic
    /// Updates camera position
    pub fn move_camera(&mut self, camera: &mut Camera, duration: Duration) {
        prof!(_guard, "Camera::move_camera");

        let dur = duration.as_secs_f32();
        let move_modifier = Self::SPEED * dur;

        // Common calculations
        let forward = camera.forward_xy();
        let right = forward.cross(F32x3::Y);

        // Move forward/backward
        camera.f_pos += forward * (self.forward - self.backward) * move_modifier;
        // Move left/right
        camera.f_pos += right * (self.left - self.right) * move_modifier;
        // Move up/down
        camera.f_pos.y += (self.up - self.down) * move_modifier;
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
        }
    }
}
