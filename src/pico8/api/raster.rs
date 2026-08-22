//! Pixel plots for Pico-8 shape primitives. Colors are baked as sRGB bytes
//! (sprite tint round-trips through linear and lands 1/255 dark).

use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub(crate) struct Raster {
    pub image: Image,
    pub pen: [u8; 4],
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

    pub fn plot(&mut self, x: i32, y: i32) {
        let size = self.image.size();
        assert!(
            x >= 0 && y >= 0 && (x as u32) < size.x && (y as u32) < size.y,
            "plot ({x}, {y}) out of raster {size}"
        );
        if let Ok(bytes) = self
            .image
            .pixel_bytes_mut(UVec3::new(x as u32, y as u32, 0))
        {
            bytes.copy_from_slice(&self.pen);
        }
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32) {
        let (lo, hi) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        for x in lo..=hi {
            self.plot(x, y);
        }
    }

    pub fn vline(&mut self, y0: i32, y1: i32, x: i32) {
        let (lo, hi) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        for y in lo..=hi {
            self.plot(x, y);
        }
    }

    /// Midpoint circle outline. Ported from fake-08 `Graphics::circ`.
    pub fn circ(&mut self, ox: i32, oy: i32, r: i32) {
        let mut x = r;
        let mut y = 0;
        let mut decision_over_2 = 1 - x;

        while y <= x {
            self.plot(ox + x, oy + y);
            self.plot(ox + y, oy + x);
            self.plot(ox - x, oy + y);
            self.plot(ox - y, oy + x);
            self.plot(ox - x, oy - y);
            self.plot(ox - y, oy - x);
            self.plot(ox + x, oy - y);
            self.plot(ox + y, oy - x);

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
        if r == 0 {
            self.plot(ox, oy);
        } else if r == 1 {
            self.plot(ox, oy - 1);
            self.hline(ox - 1, ox + 1, oy);
            self.plot(ox, oy + 1);
        } else if r > 0 {
            let mut x = -r;
            let mut y = 0;
            let mut err = 2 - 2 * r;
            loop {
                self.hline(ox - x, ox + x, oy + y);
                self.hline(ox - x, ox + x, oy - y);
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

        self.plot(xc, yc + yr);
        self.plot(xc, yc - yr);

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
            self.plot(xc + wx, yc - wy);
            self.plot(xc - wx, yc - wy);
            self.plot(xc + wx, yc + wy);
            self.plot(xc - wx, yc + wy);
        }

        self.plot(xc + xr, yc);
        self.plot(xc - xr, yc);

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
            self.plot(xc + wx, yc - wy);
            self.plot(xc - wx, yc - wy);
            self.plot(xc + wx, yc + wy);
            self.plot(xc - wx, yc + wy);
        }
    }

    /// Filled ellipse. Ported from fake-08 `Graphics::ovalfill`.
    pub fn ovalfill(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32) {
        sort_rect(&mut x0, &mut y0, &mut x1, &mut y1);
        let xr = (x1 - x0) / 2;
        let yr = (y1 - y0) / 2;
        let xc = x0 + xr;
        let yc = y0 + yr;

        self.vline(yc + yr, yc - yr, xc);

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
            self.hline(xc + wx, xc - wx, yc - wy);
            self.hline(xc + wx, xc - wx, yc + wy);
        }

        self.hline(xc + xr, xc - xr, yc);

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
            self.hline(xc + wx, xc - wx, yc - wy);
            self.hline(xc + wx, xc - wx, yc + wy);
        }
    }
}

impl From<Raster> for Image {
    fn from(raster: Raster) -> Self {
        raster.image
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
