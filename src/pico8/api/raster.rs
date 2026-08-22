//! Pixel plots for Pico-8 shape primitives. Colors are baked as sRGB bytes
//! (sprite tint round-trips through linear and lands 1/255 dark).
//!
//! Drawing walks the CPU buffer directly. Horizontal runs are a contiguous
//! RGBA slice; diagonals use the `bresenham` iterator in one loop.

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

const BPP: usize = 4;

pub struct Raster {
    pub image: Image,
    pub pen: [u8; 4],
}

/// Pico-8 `line` includes both endpoints. The `bresenham` crate does not include `end`.
pub(crate) fn bresenham_inclusive(
    start: (isize, isize),
    end: (isize, isize),
) -> impl Iterator<Item = (isize, isize)> {
    // bresenham::Bresenham::new(start, end).chain(core::iter::once(end))
    bresenham::Bresenham::new(start, end)
}

impl Raster {
    pub fn new(size: UVec2, pen: [u8; 4]) -> Self {
        let mut image = Image::new_fill(
            Extent3d {
                width: size.x.max(1),
                height: size.y.max(1),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0u8, 0u8, 0u8, 0u8],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        image.sampler = ImageSampler::nearest();
        Self { image, pen }
    }

    fn buf(&mut self) -> (&mut [u8], UVec2) {
        let size = self.image.size();
        let data = self
            .image
            .data
            .as_mut()
            .expect("raster image has CPU data")
            .as_mut_slice();
        (data, size)
    }

    #[allow(dead_code)]
    pub fn plot(&mut self, x: i32, y: i32) {
        let pen = self.pen;
        let (data, size) = self.buf();
        put(data, size, x, y, pen);
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32) {
        let pen = self.pen;
        let (data, size) = self.buf();
        fill_hline(data, size, x0, x1, y, pen);
    }

    pub fn vline(&mut self, y0: i32, y1: i32, x: i32) {
        let pen = self.pen;
        let (data, size) = self.buf();
        fill_vline(data, size, x, y0, y1, pen);
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        if y0 == y1 {
            self.hline(x0, x1, y0);
            return;
        }
        if x0 == x1 {
            self.vline(y0, y1, x0);
            return;
        }
        let pen = self.pen;
        let (data, size) = self.buf();
        for (x, y) in bresenham_inclusive((x0 as isize, y0 as isize), (x1 as isize, y1 as isize)) {
            put(data, size, x as i32, y as i32, pen);
        }
    }

    /// Midpoint circle outline. Ported from fake-08 `Graphics::circ`.
    pub fn circ(&mut self, ox: i32, oy: i32, r: i32) {
        let pen = self.pen;
        let (data, size) = self.buf();
        let mut x = r;
        let mut y = 0;
        let mut decision_over_2 = 1 - x;

        while y <= x {
            put(data, size, ox + x, oy + y, pen);
            put(data, size, ox + y, oy + x, pen);
            put(data, size, ox - x, oy + y, pen);
            put(data, size, ox - y, oy + x, pen);
            put(data, size, ox - x, oy - y, pen);
            put(data, size, ox - y, oy - x, pen);
            put(data, size, ox + x, oy - y, pen);
            put(data, size, ox + y, oy - x, pen);

            y += 1;
            if decision_over_2 < 0 {
                decision_over_2 += 2 * y + 1;
            } else {
                x -= 1;
                decision_over_2 += 2 * (y - x) + 1;
            }
        }
    }

    /// Filled circle. Ported from fake-08 `Graphics::circfill`.
    pub fn circfill(&mut self, ox: i32, oy: i32, r: i32) {
        let pen = self.pen;
        let (data, size) = self.buf();
        if r == 0 {
            put(data, size, ox, oy, pen);
        } else if r == 1 {
            put(data, size, ox, oy - 1, pen);
            fill_hline(data, size, ox - 1, ox + 1, oy, pen);
            put(data, size, ox, oy + 1, pen);
        } else if r > 0 {
            let mut x = -r;
            let mut y = 0;
            let mut err = 2 - 2 * r;
            loop {
                fill_hline(data, size, ox - x, ox + x, oy + y, pen);
                fill_hline(data, size, ox - x, ox + x, oy - y, pen);
                let saved = err;
                if saved > x {
                    x += 1;
                    err += x * 2 + 1;
                }
                if saved <= y {
                    y += 1;
                    err += y * 2 + 1;
                }
                if x >= 0 {
                    break;
                }
            }
        }
    }

    /// Ellipse outline. Ported from fake-08 `Graphics::oval` (midpoint ellipse).
    pub fn oval(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32) {
        sort_rect(&mut x0, &mut y0, &mut x1, &mut y1);
        let xr = (x1 - x0) / 2;
        let yr = (y1 - y0) / 2;
        let xc = x0 + xr;
        let yc = y0 + yr;
        let pen = self.pen;
        let (data, size) = self.buf();

        put(data, size, xc, yc + yr, pen);
        put(data, size, xc, yc - yr, pen);

        let asq = xr * xr;
        let bsq = yr * yr;
        let mut wx = 0;
        let mut wy = yr;
        let mut xa = 0;
        let mut ya = asq * 2 * yr;
        let mut thresh = asq / 4 - asq * yr;

        loop {
            thresh += xa + bsq;
            if thresh >= 0 {
                ya -= asq * 2;
                thresh -= ya;
                wy -= 1;
            }
            xa += bsq * 2;
            wx += 1;
            if xa >= ya {
                break;
            }
            put(data, size, xc + wx, yc - wy, pen);
            put(data, size, xc - wx, yc - wy, pen);
            put(data, size, xc + wx, yc + wy, pen);
            put(data, size, xc - wx, yc + wy, pen);
        }

        put(data, size, xc + xr, yc, pen);
        put(data, size, xc - xr, yc, pen);

        wx = xr;
        wy = 0;
        xa = bsq * 2 * xr;
        ya = 0;
        thresh = bsq / 4 - bsq * xr;

        loop {
            thresh += ya + asq;
            if thresh >= 0 {
                xa -= bsq * 2;
                thresh -= xa;
                wx -= 1;
            }
            ya += asq * 2;
            wy += 1;
            if ya > xa || (ya == 0 && xa == 0) {
                break;
            }
            put(data, size, xc + wx, yc - wy, pen);
            put(data, size, xc - wx, yc - wy, pen);
            put(data, size, xc + wx, yc + wy, pen);
            put(data, size, xc - wx, yc + wy, pen);
        }
    }

    /// Filled ellipse. Ported from fake-08 `Graphics::ovalfill`.
    pub fn ovalfill(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32) {
        sort_rect(&mut x0, &mut y0, &mut x1, &mut y1);
        let xr = (x1 - x0) / 2;
        let yr = (y1 - y0) / 2;
        let xc = x0 + xr;
        let yc = y0 + yr;
        let pen = self.pen;
        let (data, size) = self.buf();

        fill_vline(data, size, xc, yc + yr, yc - yr, pen);

        let asq = xr * xr;
        let bsq = yr * yr;
        let mut wx = 0;
        let mut wy = yr;
        let mut xa = 0;
        let mut ya = asq * 2 * yr;
        let mut thresh = asq / 4 - asq * yr;

        loop {
            thresh += xa + bsq;
            if thresh >= 0 {
                ya -= asq * 2;
                thresh -= ya;
                wy -= 1;
            }
            xa += bsq * 2;
            wx += 1;
            if xa >= ya {
                break;
            }
            fill_hline(data, size, xc + wx, xc - wx, yc - wy, pen);
            fill_hline(data, size, xc + wx, xc - wx, yc + wy, pen);
        }

        fill_hline(data, size, xc + xr, xc - xr, yc, pen);

        wx = xr;
        wy = 0;
        xa = bsq * 2 * xr;
        ya = 0;
        thresh = bsq / 4 - bsq * xr;

        loop {
            thresh += ya + asq;
            if thresh >= 0 {
                xa -= bsq * 2;
                thresh -= xa;
                wx -= 1;
            }
            ya += asq * 2;
            wy += 1;
            if ya > xa || (ya == 0 && xa == 0) {
                break;
            }
            fill_hline(data, size, xc + wx, xc - wx, yc - wy, pen);
            fill_hline(data, size, xc + wx, xc - wx, yc + wy, pen);
        }
    }
}

impl From<Raster> for Image {
    fn from(raster: Raster) -> Self {
        raster.image
    }
}

#[inline(always)]
fn offset(size: UVec2, x: i32, y: i32) -> usize {
    ((y as u32 * size.x + x as u32) as usize) * BPP
}

#[inline(always)]
fn put(data: &mut [u8], size: UVec2, x: i32, y: i32, pen: [u8; 4]) {
    assert!(
        x >= 0 && y >= 0 && (x as u32) < size.x && (y as u32) < size.y,
        "plot ({x}, {y}) out of raster {size}"
    );
    let i = offset(size, x, y);
    data[i..i + BPP].copy_from_slice(&pen);
}

#[inline(always)]
fn fill_hline(data: &mut [u8], size: UVec2, x0: i32, x1: i32, y: i32, pen: [u8; 4]) {
    let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
    assert!(
        y >= 0 && (y as u32) < size.y && lo >= 0 && (hi as u32) < size.x,
        "hline ({lo}..={hi}, {y}) out of raster {size}"
    );
    let start = offset(size, lo, y);
    let row = &mut data[start..start + ((hi - lo + 1) as usize) * BPP];
    for px in row.chunks_exact_mut(BPP) {
        px.copy_from_slice(&pen);
    }
}

#[inline(always)]
fn fill_vline(data: &mut [u8], size: UVec2, x: i32, y0: i32, y1: i32, pen: [u8; 4]) {
    let (lo, hi) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
    assert!(
        x >= 0 && (x as u32) < size.x && lo >= 0 && (hi as u32) < size.y,
        "vline ({x}, {lo}..={hi}) out of raster {size}"
    );
    let stride = size.x as usize * BPP;
    let mut i = offset(size, x, lo);
    for _ in lo..=hi {
        data[i..i + BPP].copy_from_slice(&pen);
        i += stride;
    }
}

fn sort_rect(x0: &mut i32, y0: &mut i32, x1: &mut i32, y1: &mut i32) {
    if x0 > x1 {
        std::mem::swap(x0, x1);
    }
    if y0 > y1 {
        std::mem::swap(y0, y1);
    }
}
