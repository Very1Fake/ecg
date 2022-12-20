use std::ops::Mul;

use glam::{IVec3, Vec3};

use crate::direction::Direction;

pub type GlobalUnit = i32;
pub type LocalUnit = u8;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SQUARE: usize = (CHUNK_SIZE).pow(2);
pub const CHUNK_CUBE: usize = (CHUNK_SIZE).pow(3);

pub const G_CHUNK_SIZE: GlobalUnit = CHUNK_SIZE as GlobalUnit;
pub const G_CHUNK_SQUARE: GlobalUnit = CHUNK_SQUARE as GlobalUnit;
pub const G_CHUNK_CUBE: GlobalUnit = CHUNK_CUBE as GlobalUnit;

pub const L_CHUNK_SIZE: LocalUnit = CHUNK_SIZE as LocalUnit;
pub const L_CHUNK_SQUARE: LocalUnit = CHUNK_SQUARE as LocalUnit;
pub const L_CHUNK_CUBE: LocalUnit = CHUNK_CUBE as LocalUnit;

////////////////////////////////////////////////////////////////////////////////////////////////////

macro_rules! coord_base_impl {
    ($repr:tt, $($T:ty),+) => {
        $(
            impl $T {
                pub const ZERO: Self = <$T>::from_vec(IVec3::ZERO);

                pub const fn new(x: $repr, y: $repr, z: $repr) -> Self {
                    Self(IVec3::new(x, y, z))
                }

                pub const fn from_vec(vec: IVec3) -> Self {
                    Self(vec)
                }

                pub fn from_float_vec(vec: Vec3) -> Self {
                    Self(vec.as_ivec3())
                }

                pub fn as_vec(&self) -> Vec3 {
                    self.0.as_vec3()
                }

                pub const fn neighbor(&self, dir: Direction) -> Self {
                    let mut new = *self;

                    match dir {
                        Direction::Down => new.0.y -= 1,
                        Direction::Up => new.0.y += 1,
                        Direction::Left => new.0.x -= 1,
                        Direction::Right => new.0.x += 1,
                        Direction::Front => new.0.z -= 1,
                        Direction::Back => new.0.z += 1,
                    }

                    new
                }
            }

            impl Default for $T {
                fn default() -> Self {
                    Self::ZERO
                }
            }
        )+
    };
}

coord_base_impl!(GlobalUnit, ChunkId, ChunkCoord, GlobalCoord);
// coord_base_impl!(LocalUnit, BlockCoord);

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents chunk id
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ChunkId(pub IVec3);

impl ChunkId {
    pub fn to_coord(&self) -> ChunkCoord {
        ChunkCoord::from_vec(self.0 * G_CHUNK_SIZE)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the coordinates of a chunk in a world
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ChunkCoord(pub IVec3);

impl ChunkCoord {
    pub fn new_checked(x: GlobalUnit, y: GlobalUnit, z: GlobalUnit) -> Self {
        Self(IVec3::new(
            x.div_euclid(G_CHUNK_SIZE).mul(G_CHUNK_SIZE),
            y.div_euclid(G_CHUNK_SIZE).mul(G_CHUNK_SIZE),
            z.div_euclid(G_CHUNK_SIZE).mul(G_CHUNK_SIZE),
        ))
    }

    pub fn to_id(&self) -> ChunkId {
        ChunkId::from_vec(self.0 / G_CHUNK_SIZE)
    }

    pub fn to_global(&self, block: &BlockCoord) -> GlobalCoord {
        GlobalCoord::from_vec(
            self.0
                + IVec3::new(
                    block.x as GlobalUnit,
                    block.y as GlobalUnit,
                    block.z as GlobalUnit,
                ),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the local coordinates of a block in a chunk
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct BlockCoord {
    pub x: LocalUnit,
    pub y: LocalUnit,
    pub z: LocalUnit,
}

impl BlockCoord {
    pub const ZERO: Self = Self::new(0, 0, 0);

    pub const fn new(x: LocalUnit, y: LocalUnit, z: LocalUnit) -> Self {
        Self { x, y, z }
    }

    // pub const fn from_vec(vec: IVec3) -> Self {
    //     Self {
    //         x: vec.x as LocalUnit,
    //         y: vec.y as LocalUnit,
    //         z: vec.z as LocalUnit,
    //     }
    // }

    // pub const fn from_float_vec(vec: Vec3) -> Self {
    //     Self {
    //         x: vec.x as LocalUnit,
    //         y: vec.y as LocalUnit,
    //         z: vec.z as LocalUnit,
    //     }
    // }

    pub fn as_vec(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }

    pub const fn neighbor(&self, dir: Direction) -> Self {
        let mut new = *self;

        match dir {
            Direction::Down => new.y -= 1,
            Direction::Up => new.y += 1,
            Direction::Left => new.x -= 1,
            Direction::Right => new.x += 1,
            Direction::Front => new.z -= 1,
            Direction::Back => new.z += 1,
        }

        new
    }

    pub fn on_chunk_edge(&self, dir: Direction) -> bool {
        match dir {
            Direction::Down => self.y == 0,
            Direction::Up => self.y == L_CHUNK_SIZE - 1,
            Direction::Left => self.x == 0,
            Direction::Right => self.x == L_CHUNK_SIZE - 1,
            Direction::Front => self.z == 0,
            Direction::Back => self.z == L_CHUNK_SIZE - 1,
        }
    }

    pub fn flatten(&self) -> usize {
        (self.x as usize).mul(CHUNK_SQUARE) + (self.y as usize).mul(CHUNK_SIZE) + self.z as usize
    }
}

impl Default for BlockCoord {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<usize> for BlockCoord {
    fn from(idx: usize) -> Self {
        Self {
            x: idx.div_euclid(CHUNK_SQUARE) as LocalUnit,
            y: idx.rem_euclid(CHUNK_SQUARE).div_euclid(CHUNK_SIZE) as LocalUnit,
            z: idx.rem_euclid(CHUNK_SIZE) as LocalUnit,
        }
    }
}

impl From<GlobalUnit> for BlockCoord {
    fn from(idx: GlobalUnit) -> Self {
        Self {
            x: idx.div_euclid(G_CHUNK_SQUARE) as LocalUnit,
            y: idx.rem_euclid(G_CHUNK_SQUARE).div_euclid(G_CHUNK_SIZE) as LocalUnit,
            z: idx.rem_euclid(G_CHUNK_SIZE) as LocalUnit,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the coordinates of a block in the world
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct GlobalCoord(pub IVec3);

impl GlobalCoord {
    pub fn to_chunk_id(&self) -> ChunkId {
        ChunkId::from_vec(self.0 / G_CHUNK_SIZE)
    }

    pub fn to_chunk(&self) -> ChunkCoord {
        ChunkCoord::from_vec((self.0 / G_CHUNK_SIZE) * G_CHUNK_SIZE)
    }

    pub fn to_block(&self) -> BlockCoord {
        BlockCoord::new(
            (self.0.x as LocalUnit).rem_euclid(L_CHUNK_SIZE),
            (self.0.y as LocalUnit).rem_euclid(L_CHUNK_SIZE),
            (self.0.z as LocalUnit).rem_euclid(L_CHUNK_SIZE),
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    //! All tests written assuming that chunk size is 16 blocks
    use super::{BlockCoord, ChunkCoord, ChunkId, GlobalCoord};

    #[test]
    fn block_from_usize() {
        assert_eq!(BlockCoord::from(0_usize), BlockCoord::ZERO);
        assert_eq!(BlockCoord::from(291_usize), BlockCoord::new(1, 2, 3));
        assert_eq!(BlockCoord::from(801_usize), BlockCoord::new(3, 2, 1));
        assert_eq!(BlockCoord::from(4104_usize), BlockCoord::new(16, 0, 8));
    }

    #[test]
    fn global_to_chunk_id() {
        assert_eq!(GlobalCoord::ZERO.to_chunk_id(), ChunkId::ZERO);
        assert_eq!(GlobalCoord::new(15, 15, 15).to_chunk_id(), ChunkId::ZERO);
        assert_eq!(
            GlobalCoord::new(31, 31, 31).to_chunk_id(),
            ChunkId::new(1, 1, 1)
        );
        assert_eq!(
            GlobalCoord::new(127, 31, 256).to_chunk_id(),
            ChunkId::new(7, 1, 16)
        );
    }

    #[test]
    fn global_to_chunk() {
        assert_eq!(GlobalCoord::ZERO.to_chunk(), ChunkCoord::ZERO);
        assert_eq!(GlobalCoord::new(15, 15, 15).to_chunk(), ChunkCoord::ZERO);
        assert_eq!(
            GlobalCoord::new(31, 31, 31).to_chunk(),
            ChunkCoord::new(16, 16, 16)
        );
        assert_eq!(
            GlobalCoord::new(127, 31, 256).to_chunk(),
            ChunkCoord::new(112, 16, 256)
        );
    }

    #[test]
    fn global_to_block() {
        assert_eq!(GlobalCoord::ZERO.to_block(), BlockCoord::ZERO);
        assert_eq!(
            GlobalCoord::new(15, 15, 15).to_block(),
            BlockCoord::new(15, 15, 15)
        );
        assert_eq!(
            GlobalCoord::new(31, 31, 31).to_block(),
            BlockCoord::new(15, 15, 15)
        );
        assert_eq!(
            GlobalCoord::new(127, 31, 256).to_block(),
            BlockCoord::new(15, 15, 0)
        );
        assert_eq!(
            GlobalCoord::new(156, 33, 264).to_block(),
            BlockCoord::new(12, 1, 8)
        );
    }
}
