pub type BlockRepr = u8;

/// Represents block ID
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum Block {
    #[default]
    Air = 0,
    Stone = 1,
    Grass = 2,
    Sand = 3,
}

impl Block {
    pub const MIN: BlockRepr = 0;
    pub const MAX: BlockRepr = Self::Sand as BlockRepr;

    pub fn id(&self) -> BlockRepr {
        *self as BlockRepr
    }
}

impl From<BlockRepr> for Block {
    fn from(id: BlockRepr) -> Self {
        match id {
            0 => Self::Air,
            1 => Self::Stone,
            2 => Self::Grass,
            3 => Self::Sand,
            _ => Self::Air,
        }
    }
}
