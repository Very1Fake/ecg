use common::direction::Direction;

use crate::types::F32x3;

pub const HALF_SIZE: f32 = 0.5;

/// Helper struct for building quad meshes
#[derive(Debug)]
pub struct Quad {
    pub direction: Direction,
    pub position: F32x3,
}

impl Quad {
    pub const LEFT_UP_FRONT: F32x3 = F32x3::new(-HALF_SIZE, HALF_SIZE, -HALF_SIZE);
    pub const LEFT_UP_BACK: F32x3 = F32x3::new(-HALF_SIZE, HALF_SIZE, HALF_SIZE);
    pub const LEFT_DOWN_FRONT: F32x3 = F32x3::new(-HALF_SIZE, -HALF_SIZE, -HALF_SIZE);
    pub const LEFT_DOWN_BACK: F32x3 = F32x3::new(-HALF_SIZE, -HALF_SIZE, HALF_SIZE);

    pub const RIGHT_UP_FRONT: F32x3 = F32x3::new(HALF_SIZE, HALF_SIZE, -HALF_SIZE);
    pub const RIGHT_UP_BACK: F32x3 = F32x3::new(HALF_SIZE, HALF_SIZE, HALF_SIZE);
    pub const RIGHT_DOWN_FRONT: F32x3 = F32x3::new(HALF_SIZE, -HALF_SIZE, -HALF_SIZE);
    pub const RIGHT_DOWN_BACK: F32x3 = F32x3::new(HALF_SIZE, -HALF_SIZE, HALF_SIZE);

    pub fn new(direction: Direction, position: F32x3) -> Self {
        Self {
            direction,
            position,
        }
    }

    /// Get quad corners (vertices positions)
    pub fn corners(&self) -> [F32x3; 4] {
        let pos = self.position;

        match self.direction {
            Direction::Down => [
                Self::RIGHT_DOWN_FRONT + pos,
                Self::RIGHT_DOWN_BACK + pos,
                Self::LEFT_DOWN_BACK + pos,
                Self::LEFT_DOWN_FRONT + pos,
            ],
            Direction::Up => [
                Self::RIGHT_UP_BACK + pos,
                Self::RIGHT_UP_FRONT + pos,
                Self::LEFT_UP_FRONT + pos,
                Self::LEFT_UP_BACK + pos,
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
