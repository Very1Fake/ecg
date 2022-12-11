use glam::Vec3;

pub type BlockRepr = u8;

/// Represents block ID
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum Block {
    Void = 0,

    #[default]
    Air,

    // Basic
    Stone,
    Dirt,
    Grass,
    Leaves,

    // Liquid
    Water,
    MovingWater,
    Magma,
    MovingMagma,
    Lava,
    MovingLava,

    // Hot Biomes
    SandStone,
    Sand,

    // Temperate Biomes
    Clay,
    Mud,

    // Cold Biomes
    SnowBlock,
    Ice,
}

impl Block {
    pub const MIN: BlockRepr = Self::Void as BlockRepr;
    pub const MAX: BlockRepr = Self::Ice as BlockRepr;

    pub const ALL: [Self; 18] = [
        Self::Void,
        Self::Air,
        Self::Stone,
        Self::Dirt,
        Self::Grass,
        Self::Leaves,
        Self::Water,
        Self::MovingWater,
        Self::Magma,
        Self::MovingMagma,
        Self::Lava,
        Self::MovingLava,
        Self::SandStone,
        Self::Sand,
        Self::Clay,
        Self::Mud,
        Self::SnowBlock,
        Self::Ice,
    ];

    pub fn id(&self) -> BlockRepr {
        *self as BlockRepr
    }

    #[inline]
    pub fn opaque(&self) -> bool {
        !matches!(self, Self::Air | Self::Void)
    }

    #[inline]
    pub fn liquid(&self) -> bool {
        matches!(
            self,
            Self::Water
                | Self::MovingWater
                | Self::Magma
                | Self::MovingMagma
                | Self::Lava
                | Self::MovingLava
        )
    }

    pub fn color(&self) -> Vec3 {
        match self {
            Self::Void => Vec3::new(0.0, 0.0, 0.0),
            Self::Air => Vec3::new(1.0, 1.0, 1.0),
            Self::Stone => Vec3::new(0.525, 0.53, 0.52),
            Self::Dirt => Vec3::new(0.28, 0.16, 0.047),
            Self::Grass => Vec3::new(0.189, 0.82, 0.378),
            Self::Leaves => Vec3::new(0.104, 0.69, 0.367),
            Self::Water => Vec3::new(0.0456, 0.593, 0.76),
            Self::MovingWater => Vec3::new(0.0456, 0.593, 0.76),
            Self::Magma => Vec3::new(0.89, 0.0534, 0.0534),
            Self::MovingMagma => Vec3::new(0.89, 0.0534, 0.0534),
            Self::Lava => Vec3::new(1.00, 0.348, 0.15),
            Self::MovingLava => Vec3::new(1.00, 0.348, 0.15),
            Self::SandStone => Vec3::new(0.76, 0.755, 0.464),
            Self::Sand => Vec3::new(0.82, 0.815, 0.533),
            Self::Clay => Vec3::new(0.691, 0.7, 0.609),
            Self::Mud => Vec3::new(0.17, 0.131, 0.0221),
            Self::SnowBlock => Vec3::new(0.98, 0.98, 0.98),
            Self::Ice => Vec3::new(0.747, 0.877, 0.97),
        }
    }
}

impl From<BlockRepr> for Block {
    fn from(id: BlockRepr) -> Self {
        match id {
            0 => Self::Void,
            1 => Self::Air,
            2 => Self::Stone,
            3 => Self::Dirt,
            4 => Self::Grass,
            5 => Self::Leaves,
            6 => Self::Water,
            7 => Self::MovingWater,
            8 => Self::Magma,
            9 => Self::MovingMagma,
            10 => Self::Lava,
            11 => Self::MovingLava,
            12 => Self::SandStone,
            13 => Self::Sand,
            14 => Self::Clay,
            15 => Self::Mud,
            16 => Self::SnowBlock,
            17 => Self::Ice,
            _ => Self::Void,
        }
    }
}
