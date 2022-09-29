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
    pub const LEFT_UP_FRONT: Float32x3 = Float32x3::new(-HALF_SIZE, HALF_SIZE, -HALF_SIZE);
    pub const LEFT_UP_BACK: Float32x3 = Float32x3::new(-HALF_SIZE, HALF_SIZE, HALF_SIZE);
    pub const LEFT_DOWN_FRONT: Float32x3 = Float32x3::new(-HALF_SIZE, -HALF_SIZE, -HALF_SIZE);
    pub const LEFT_DOWN_BACK: Float32x3 = Float32x3::new(-HALF_SIZE, -HALF_SIZE, HALF_SIZE);

    pub const RIGHT_UP_FRONT: Float32x3 = Float32x3::new(HALF_SIZE, HALF_SIZE, -HALF_SIZE);
    pub const RIGHT_UP_BACK: Float32x3 = Float32x3::new(HALF_SIZE, HALF_SIZE, HALF_SIZE);
    pub const RIGHT_DOWN_FRONT: Float32x3 = Float32x3::new(HALF_SIZE, -HALF_SIZE, -HALF_SIZE);
    pub const RIGHT_DOWN_BACK: Float32x3 = Float32x3::new(HALF_SIZE, -HALF_SIZE, HALF_SIZE);

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
                Self::RIGHT_UP_BACK + pos,
                Self::RIGHT_UP_FRONT + pos,
                Self::LEFT_UP_FRONT + pos,
                Self::LEFT_UP_BACK + pos,
            ],
            Direction::Down => [
                Self::RIGHT_DOWN_FRONT + pos,
                Self::RIGHT_DOWN_BACK + pos,
                Self::LEFT_DOWN_BACK + pos,
                Self::LEFT_DOWN_FRONT + pos,
            ],
            Direction::Left => [
                Self::LEFT_UP_FRONT + pos,
                Self::LEFT_DOWN_FRONT + pos,
                Self::LEFT_DOWN_BACK + pos,
                Self::LEFT_UP_BACK + pos,
            ],
            Direction::Right => [
                Self::RIGHT_UP_BACK + pos,
                Self::RIGHT_DOWN_BACK + pos,
                Self::RIGHT_DOWN_FRONT + pos,
                Self::RIGHT_UP_FRONT + pos,
            ],
            Direction::Front => [
                Self::RIGHT_UP_FRONT + pos,
                Self::RIGHT_DOWN_FRONT + pos,
                Self::LEFT_DOWN_FRONT + pos,
                Self::LEFT_UP_FRONT + pos,
            ],
            Direction::Back => [
                Self::LEFT_UP_BACK + pos,
                Self::LEFT_DOWN_BACK + pos,
                Self::RIGHT_DOWN_BACK + pos,
                Self::RIGHT_UP_BACK + pos,
            ],
        }
    }
}
