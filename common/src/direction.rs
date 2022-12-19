#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Down,
    Up,
    Left,
    Right,
    Front,
    Back,
}

impl Direction {
    pub const ALL: [Self; 6] = [
        Self::Down,
        Self::Up,
        Self::Left,
        Self::Right,
        Self::Front,
        Self::Back,
    ];

    pub const fn reverse(&self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Front => Self::Back,
            Self::Back => Self::Front,
        }
    }
}
