use glam::Vec3;

pub type Unit = usize; // FIX

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SQUARE: usize = CHUNK_SIZE.pow(2);
pub const CHUNK_CUBE: usize = CHUNK_SIZE.pow(3);

pub trait FlattenedCoord {
    fn flatten(&self) -> Unit;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

macro_rules! coord_base_impl {
    ($($T:ty),+) => {
        $(
            impl $T {
                pub const ZERO: Self = <$T>::new(0, 0, 0);

                pub const fn new(x: Unit, y: Unit, z: Unit) -> Self {
                    Self { x, y, z }
                }

                pub fn as_vec(&self) -> Vec3 {
                    Vec3::new(self.x as f32, self.y as f32, self.z as f32)
                }
            }
        )+
    };
}

coord_base_impl!(ChunkCoord, BlockCoord, GlobalCoord);

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the coordinates of a chunk in a world
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ChunkCoord {
    x: Unit,
    y: Unit,
    z: Unit,
}

impl ChunkCoord {
    pub fn into_global(&self, block: &BlockCoord) -> GlobalCoord {
        GlobalCoord::new(self.x + block.x, self.y + block.y, self.z + block.z)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the local coordinates of a block in a chunk
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct BlockCoord {
    x: Unit,
    y: Unit,
    z: Unit,
}

impl FlattenedCoord for BlockCoord {
    fn flatten(&self) -> Unit {
        self.z * CHUNK_SQUARE + self.y * CHUNK_SIZE + self.x
    }
}

impl From<usize> for BlockCoord {
    fn from(idx: usize) -> Self {
        Self {
            x: idx / (CHUNK_SQUARE),
            y: idx % (CHUNK_SQUARE) / CHUNK_SIZE,
            z: idx % CHUNK_SIZE,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents the coordinates of a block in the world
#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct GlobalCoord {
    x: Unit,
    y: Unit,
    z: Unit,
}

#[cfg(test)]
mod tests {
    use super::BlockCoord;

    #[test]
    fn block_coords_from_usize() {
        assert_eq!(BlockCoord::from(0), BlockCoord::ZERO);
        assert_eq!(BlockCoord::from(291), BlockCoord::new(1, 2, 3));
        assert_eq!(BlockCoord::from(801), BlockCoord::new(3, 2, 1));
        assert_eq!(BlockCoord::from(4104), BlockCoord::new(16, 0, 8));
    }
}
