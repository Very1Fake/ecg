use std::f32::consts::FRAC_PI_2;

use criterion::{criterion_group, criterion_main, Criterion};

use ecg_game::types::{F32x2, F32x3, Mat4};

pub fn view_mat_bench(c: &mut Criterion) {
    let pos = F32x3::new(5.0, 0.5, 0.0);
    let rot = F32x2::new(-FRAC_PI_2, 0.08333);
    let dist = 5.0;
    let mode = OldCameraMode::ThirdPerson { target: pos };

    // Old camera view matrix function
    let old = || match mode {
        OldCameraMode::FirstPerson { forward } => Mat4::look_to_lh(pos, forward, F32x3::Y),
        OldCameraMode::ThirdPerson { target } => Mat4::look_at_lh(pos, target, F32x3::Y),
    };
    
    // Current camera view matrix function
    let new = || {
        Mat4::from_translation(F32x3::new(0.0, 0.0, dist))
            * Mat4::from_rotation_x(-rot.y)
            * Mat4::from_rotation_y(-rot.x)
            * Mat4::from_translation(-pos)
    };

    let mut group = c.benchmark_group("Camera View Matrix");

    group.bench_function("old", |b| b.iter(|| old()));
    group.bench_function("new", |b| b.iter(|| new()));

    group.finish();
}

// Old camera mode enum,
// used to reproduce old view matrix function
pub enum OldCameraMode {
    FirstPerson { forward: F32x3 },
    ThirdPerson { target: F32x3 },
}

criterion_group!(benches, view_mat_bench);
criterion_main!(benches);
