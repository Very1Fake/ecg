use glam::Vec3;

pub type GlobalUnit = i64;
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
                pub const ZERO: Self = <$T>::new(0, 0, 0);

                pub const fn new(x: $repr, y: $repr, z: $repr) -> Self {
                    Self { x, y, z }
                }

                pub fn as_vec(&self) -> Vec3 {
                    Vec3::new(self.x as f32, self.y as f32, self.z as f32)
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

coord_base_impl!(GlobalUnit, ChunkCoord, GlobalCoord);
coord_base_impl!(LocalUnit, BlockCoord);

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the coordinates of a chunk in a world
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ChunkCoord {
    pub x: GlobalUnit,
    pub y: GlobalUnit,
    pub z: GlobalUnit,
}

impl ChunkCoord {
    pub fn to_global(&self, block: &BlockCoord) -> GlobalCoord {
        GlobalCoord::new(
            self.x * G_CHUNK_SIZE + block.x as GlobalUnit,
            self.y * G_CHUNK_SIZE + block.y as GlobalUnit,
            self.z * G_CHUNK_SIZE + block.z as GlobalUnit,
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
    pub fn flatten(&self) -> usize {
        self.z as usize * CHUNK_SQUARE + self.y as usize * CHUNK_SIZE + self.x as usize
    }
}

impl From<usize> for BlockCoord {
    fn from(idx: usize) -> Self {
        Self {
            x: (idx / (CHUNK_SQUARE)) as LocalUnit,
            y: (idx % (CHUNK_SQUARE) / CHUNK_SIZE) as LocalUnit,
            z: (idx % CHUNK_SIZE) as LocalUnit,
        }
    }
}

impl From<GlobalUnit> for BlockCoord {
    fn from(idx: GlobalUnit) -> Self {
        Self {
            x: (idx / (G_CHUNK_SQUARE)) as LocalUnit,
            y: (idx % (G_CHUNK_SQUARE) / G_CHUNK_SIZE) as LocalUnit,
            z: (idx % G_CHUNK_SIZE) as LocalUnit,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the coordinates of a block in the world
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct GlobalCoord {
    pub x: GlobalUnit,
    pub y: GlobalUnit,
    pub z: GlobalUnit,
}

impl GlobalCoord {
    pub fn to_chunk(&self) -> ChunkCoord {
        ChunkCoord::new(
            self.x / G_CHUNK_SIZE,
            self.y / G_CHUNK_SIZE,
            self.z / G_CHUNK_SIZE,
        )
    }

    pub fn to_block(&self) -> BlockCoord {
        BlockCoord::new(
            self.x as LocalUnit % L_CHUNK_SIZE,
            self.y as LocalUnit % L_CHUNK_SIZE,
            self.z as LocalUnit % L_CHUNK_SIZE,
        )
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    //! All tests written assuming that chunk size is 16 blocks

    use super::{BlockCoord, ChunkCoord, GlobalCoord};

    #[test]
    fn block_from_usize() {
        assert_eq!(BlockCoord::from(0_usize), BlockCoord::ZERO);
        assert_eq!(BlockCoord::from(291_usize), BlockCoord::new(1, 2, 3));
        assert_eq!(BlockCoord::from(801_usize), BlockCoord::new(3, 2, 1));
        assert_eq!(BlockCoord::from(4104_usize), BlockCoord::new(16, 0, 8));
    }

    #[test]
    fn global_to_chunk() {
        assert_eq!(GlobalCoord::ZERO.to_chunk(), ChunkCoord::ZERO);
        assert_eq!(GlobalCoord::new(15, 15, 15).to_chunk(), ChunkCoord::ZERO);
        assert_eq!(
            GlobalCoord::new(31, 31, 31).to_chunk(),
            ChunkCoord::new(1, 1, 1)
        );
        assert_eq!(
            GlobalCoord::new(127, 31, 256).to_chunk(),
            ChunkCoord::new(7, 1, 16)
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
