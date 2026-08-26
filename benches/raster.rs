//! Raster primitive benches (Pico-8 128×128 canvas).
//!
//! Baseline (this tree, before the buffer-write stash):
//! ```sh
//! cargo bench-raster --save-baseline before
//! ```
//! After restoring the stash:
//! ```sh
//! cargo bench-raster --baseline before
//! ```

use bevy::math::UVec2;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use nano9::pico8::Raster;

const PEN: [u8; 4] = [255, 241, 232, 255];
const SCREEN: UVec2 = UVec2::new(128, 128);

fn raster() -> Raster {
    Raster::new(SCREEN, PEN)
}

fn primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("raster");
    let mut r = raster();

    group.bench_function("plot_screen", |b| {
        b.iter(|| {
            for y in 0..128 {
                for x in 0..128 {
                    r.plot(x, y);
                }
            }
            black_box(&r.image);
        });
    });

    group.bench_function("hline_screen", |b| {
        b.iter(|| {
            for y in 0..128 {
                r.hline(0, 127, y);
            }
            black_box(&r.image);
        });
    });

    group.bench_function("vline_screen", |b| {
        b.iter(|| {
            for x in 0..128 {
                r.vline(0, 127, x);
            }
            black_box(&r.image);
        });
    });

    group.bench_function("line_diag", |b| {
        b.iter(|| {
            r.line(0, 0, 127, 127);
            black_box(&r.image);
        });
    });

    group.bench_function("circ_r30", |b| {
        b.iter(|| {
            r.circ(64, 64, 30);
            black_box(&r.image);
        });
    });

    group.bench_function("circfill_r30", |b| {
        b.iter(|| {
            r.circfill(64, 64, 30);
            black_box(&r.image);
        });
    });

    group.bench_function("oval", |b| {
        b.iter(|| {
            r.oval(70, 20, 120, 80);
            black_box(&r.image);
        });
    });

    group.bench_function("ovalfill", |b| {
        b.iter(|| {
            r.ovalfill(10, 20, 50, 60);
            black_box(&r.image);
        });
    });

    group.finish();
}

criterion_group!(benches, primitives);
criterion_main!(benches);
