#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

impl Direction {
    pub const ALL: [Self; 6] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::Front,
        Self::Back,
    ];
}
