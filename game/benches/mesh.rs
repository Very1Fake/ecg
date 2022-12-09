use common::{
    block::Block,
    coord::{ChunkCoord, CHUNK_CUBE, CHUNK_SIZE, CHUNK_SQUARE},
};
use criterion::{criterion_group, criterion_main, Criterion};

use ecg_game::{render::mesh::TerrainMesh, types::F32x3};

pub fn simple_mesh(c: &mut Criterion) {
    let coord = ChunkCoord::ZERO;
    let mut blocks: Box<[Block]>;

    let mut group = c.benchmark_group("Simple Mesh");

    blocks = vec![Block::Air; CHUNK_CUBE].into_boxed_slice();
    group.bench_function("empty", |b| b.iter(|| TerrainMesh::build(coord, &blocks)));

    blocks = vec![Block::Air; CHUNK_CUBE].into_boxed_slice();
    blocks[0] = Block::Stone;
    group.bench_function("first", |b| b.iter(|| TerrainMesh::build(coord, &blocks)));

    blocks = vec![Block::Air; CHUNK_CUBE].into_boxed_slice();
    blocks[CHUNK_CUBE - 1] = Block::Stone;
    group.bench_function("last", |b| b.iter(|| TerrainMesh::build(coord, &blocks)));

    blocks = vec![Block::Air; CHUNK_CUBE].into_boxed_slice();
    blocks[0] = Block::Stone; // BOTTOM FRONT LEFT
    blocks[CHUNK_SIZE - 1] = Block::Stone; // BOTTOM BACK LEFT
    blocks[CHUNK_SQUARE - CHUNK_SIZE] = Block::Stone; // TOP FRONT LEFT
    blocks[CHUNK_SQUARE - 1] = Block::Stone; // TOP BACK LEFT
    blocks[CHUNK_CUBE - CHUNK_SQUARE + CHUNK_SIZE - 1] = Block::Stone; // BOTTOM BACK RIGHT
    blocks[CHUNK_CUBE - CHUNK_SQUARE] = Block::Stone; // BOTTOM FRONT RIGHT
    blocks[CHUNK_CUBE - CHUNK_SIZE] = Block::Stone; // TOP FRONT RIGHT
    blocks[CHUNK_CUBE - 1] = Block::Stone; // TOP BACK RIGHT
    group.bench_function("corners", |b| b.iter(|| TerrainMesh::build(coord, &blocks)));

    blocks = vec![Block::Stone; CHUNK_CUBE].into_boxed_slice();
    group.bench_function("full", |b| b.iter(|| TerrainMesh::build(coord, &blocks)));

    group.finish();
}

// Old camera mode enum,
// used to reproduce old view matrix function
pub enum OldCameraMode {
    FirstPerson { forward: F32x3 },
    ThirdPerson { target: F32x3 },
}

criterion_group!(benches, simple_mesh);
criterion_main!(benches);
