use crate::types::Float32x3;

use super::direction::Direction;

pub const HALF_SIZE: f32 = 0.5;

/// Helper struct for building quad meshes
#[derive(Debug)]
pub struct Quad {
    pub direction: Direction,
    pub position: Float32x3,
}

impl Quad {
    pub fn new(direction: Direction, position: Float32x3) -> Self {
        Self {
            direction,
            position,
        }
    }

    /// Get quad corners (vertices positions)
    pub fn corners(&self) -> [Float32x3; 4] {
        let pos = self.position;

        match self.direction {
            Direction::Up => [
                // Left Up Back
                Float32x3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                // Left Up Front
                Float32x3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                // Right Up Front
                Float32x3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                // Right Up Back
                Float32x3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
            ],
            Direction::Down => [
                // Left Down Front
                Float32x3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                // Left Down Back
                Float32x3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                // Right Down Back
                Float32x3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                // Right Down Front
                Float32x3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Left => [
                // Left Up Back
                Float32x3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                // Left Down Back
                Float32x3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                // Left Down Front
                Float32x3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                // Left Up Front
                Float32x3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Right => [
                // Right Up Front
                Float32x3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                // Right Down Front
                Float32x3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                // Right Down Back
                Float32x3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                // Right Up Back
                Float32x3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
            ],
            Direction::Front => [
                // Left Up Front
                Float32x3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
                // Left Down Front
                Float32x3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                // Right Down Front
                Float32x3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z - HALF_SIZE),
                // Right Up Front
                Float32x3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z - HALF_SIZE),
            ],
            Direction::Back => [
                // Right Up Back
                Float32x3::new(pos.x + HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
                // Right Down Back
                Float32x3::new(pos.x + HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                // Left Down Back
                Float32x3::new(pos.x - HALF_SIZE, pos.y - HALF_SIZE, pos.z + HALF_SIZE),
                // Left Up Back
                Float32x3::new(pos.x - HALF_SIZE, pos.y + HALF_SIZE, pos.z + HALF_SIZE),
            ],
        }
    }
}
