//! get_pixel, set_pixel operations for Image
//!
use bevy::render::{
    render_resource::{TextureFormat, Extent3d},
    texture::{Image, TextureFormatPixelInfo}
};

// use bevy::color::{Color, ColorToComponents, ColorToPacked, LinearRgba, Srgba};
use bevy::prelude::Color;
use bevy::math::UVec2;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum PixelError {
    #[error("pixel operations not supported for compressed images")]
    CompressionNotSupported,
    #[error("pixel operations not supported for depth images")]
    DepthNotSupported,
    #[error("pixel operations not supported for stencil images")]
    StencilNotSupported,
    #[error("no such pixel location")]
    InvalidLocation,
    #[error("could not align pixel data")]
    AlignmentFailed,
    #[error("invalid range")]
    InvalidRange,
    #[error("given image width cannot cleanly divide pixel count to determine image height")]
    WidthNotDivisible
}

pub enum PixelLoc {
    Linear { index: usize },
    Cartesian { x: usize, y: usize },
}

impl PixelLoc {
    fn index(&self, extent: &Extent3d) -> Option<usize> {
        match self {
            Self::Linear { index } => {
                (*index < (extent.width * extent.height) as usize).then_some(*index)
            }
            Self::Cartesian { x, y } => (*x < extent.width as usize && *y < extent.height as usize)
                .then_some(y * extent.width as usize + x),
        }
    }
}

impl From<usize> for PixelLoc {
    fn from(index: usize) -> Self {
        Self::Linear { index }
    }
}

impl From<(usize, usize)> for PixelLoc {
    fn from((x, y): (usize, usize)) -> Self {
        Self::Cartesian { x, y }
    }
}

impl From<UVec2> for PixelLoc {
    fn from(v: UVec2) -> Self {
        Self::Cartesian {
            x: v.x as usize,
            y: v.y as usize,
        }
    }
}

pub trait PixelAccess {

    fn get_pixel(&self, location: impl Into<PixelLoc>) -> Result<Color, PixelError>;

    fn set_pixel(
        &mut self,
        location: impl Into<PixelLoc>,
        color: impl Into<Color>,
    ) -> Result<(), PixelError>;

    fn set_pixels<F: FnMut(usize, usize) -> Color>(
        &mut self,
        f: F
    ) -> Result<(), PixelError>;

    // fn from_pixels<C: Into<LinearRgba> + Copy>(colors: &[C], image_width_pixels: usize) -> Result<Self, PixelError> {
}

impl PixelAccess for Image {

    fn get_pixel(&self, location: impl Into<PixelLoc>) -> Result<Color, PixelError> {
        use TextureFormat::*;
        let image_size: Extent3d = self.texture_descriptor.size;
        let format = self.texture_descriptor.format;
        let components = format.components() as usize;
        let pixel_size = format.pixel_size();
        let start = location
            .into()
            .index(&image_size)
            .ok_or(PixelError::InvalidLocation)?;
        match format {
            // Rgba32Float => {
            //     let floats = align_to::<u8, f32>(&self.data)?;
            //     let mut a = [0.0f32; 4];
            //     a.copy_from_slice(&floats[start..start + components]);
            //     Ok(LinearRgba::from_f32_array(a).into())
            // }
            // R8Unorm => {
            //     let mut a: [u8; 4] = [0, 0, 0, u8::MAX];
            //     a[0..1].copy_from_slice(&self.data[start..start + pixel_size]);
            //     Ok(LinearRgba::from_u8_array(a).into())
            // }
            // R8Snorm => {
            //     let signed = align_to::<u8, i8>(&self.data)?;
            //     let mut a: [i8; 4] = [0, 0, 0, i8::MAX];
            //     a[0..1].copy_from_slice(&signed[start..start + pixel_size]);
            //     Ok(LinearRgba::new(
            //         a[0] as f32 / i8::MAX as f32,
            //         a[1] as f32 / i8::MAX as f32,
            //         a[2] as f32 / i8::MAX as f32,
            //         a[3] as f32 / i8::MAX as f32,
            //     )
            //     .into())
            // }
            // R8Uint => {
            //     let mut a: [u8; 4] = [0, 0, 0, u8::MAX];
            //     a[0..1].copy_from_slice(&self.data[start..start + pixel_size]);
            //     Ok(LinearRgba::new(a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32).into())
            // }
            // R8Sint => {
            //     let signed = align_to::<u8, i8>(&self.data)?;
            //     let mut a: [i8; 4] = [0, 0, 0, 127];
            //     a[0..1].copy_from_slice(&signed[start..start + pixel_size]);
            //     Ok(LinearRgba::new(a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32).into())
            // }
            Rgba8Unorm => {
                let mut a = [0u8; 4];
                a.copy_from_slice(&self.data[start..start + pixel_size]);
                // Ok(LinearRgba::from_u8_array(a).into())
                Ok(Color::rgba_linear_from_array(a.map(|x| x as f32 / u8::MAX as f32)))
            }
            Rgba8UnormSrgb => {
                let mut a = [0u8; 4];
                a.copy_from_slice(&self.data[start..start + pixel_size]);
                Ok(Color::rgba_from_array(a.map(|x| x as f32 / u8::MAX as f32)))
                // Ok(Srgba::from_u8_array(a).into())
            }
            f => {
                if f.is_compressed() {
                    Err(PixelError::CompressionNotSupported)
                } else if f.has_depth_aspect() {
                    Err(PixelError::DepthNotSupported)
                } else if f.has_stencil_aspect() {
                    Err(PixelError::StencilNotSupported)
                } else {
                    todo!("Fix {f:?}");
                }
            }
        }
    }

    fn set_pixels<F: FnMut(usize, usize) -> Color>(
        &mut self,
        mut f: F
    ) -> Result<(), PixelError> {
        let image_size: Extent3d = self.texture_descriptor.size;
        for i in 0..image_size.width as usize {
            for j in 0..image_size.height as usize {
                self.set_pixel((i,j), f(i,j))?;
            }
        }
        Ok(())
    }

    fn set_pixel(
        &mut self,
        location: impl Into<PixelLoc>,
        color: impl Into<Color>,
    ) -> Result<(), PixelError> {
        use TextureFormat::*;
        let image_size: Extent3d = self.texture_descriptor.size;
        let format = self.texture_descriptor.format;
        let components = format.components() as usize;
        let color = color.into();
        let pixel_size = self.texture_descriptor.format.pixel_size();
        let start = location
            .into()
            .index(&image_size)
            .ok_or(PixelError::InvalidLocation)? * pixel_size;

        // dbg!(start);
        match format {
            // Rgba32Float => {
            //     let floats = align_to_mut::<u8, f32>(&mut self.data)?;
            //     let c: LinearRgba = color.into();
            //     let a = c.to_f32_array();
            //     floats[start..start + components].copy_from_slice(&a);
            //     Ok(())
            // }
            Rgba8Unorm => {
                // let c: LinearRgba = color.into();
                // let a = c.to_u8_array();
                let a: [u8; 4] = u32::to_le_bytes(color.as_linear_rgba_u32());
                self.data[start..start + pixel_size].copy_from_slice(&a);
                Ok(())
            }
            Rgba8UnormSrgb => {
                // let c: Srgba = color.into();
                // let a = c.to_u8_array();
                let a = color.as_rgba_u8();
                self.data[start..start + pixel_size].copy_from_slice(&a);
                Ok(())
            }
            f => {
                if f.is_compressed() {
                    Err(PixelError::CompressionNotSupported)
                } else if f.has_depth_aspect() {
                    Err(PixelError::DepthNotSupported)
                } else if f.has_stencil_aspect() {
                    Err(PixelError::StencilNotSupported)
                } else {
                    todo!("Fix {f:?}");
                }
            }
        }
    }

    // pub fn from_pixels<C: Into<LinearRgba> + Copy>(colors: &[C], image_width_pixels: usize) -> Result<Self, PixelError> {
    //     let format = TextureFormat::Rgba8Unorm;
    //     let mut data: Vec<u8> = Vec::with_capacity(colors.len() * 4);
    //     data.resize(data.capacity(), 0u8);
    //     let mut start = 0;
    //     let components = format.components() as usize;
    //     if colors.len() % image_width_pixels != 0 {
    //         return Err(PixelError::WidthNotDivisible);
    //     }
    //     let extent = Extent3d {
    //         width: image_width_pixels as u32,
    //         height: (colors.len() / image_width_pixels) as u32,
    //         depth_or_array_layers: 1,
    //     };
    //     for c in colors {
    //         let a: [u8; 4] = (*c).into().to_u8_array();
    //         data[start..start + components].copy_from_slice(&a[0..4]);
    //         start += components;
    //     }
    //     Ok(Image::new_fill(
    //         extent,
    //         TextureDimension::D2,
    //         &data,
    //         format,
    //         RenderAssetUsages::MAIN_WORLD,
    //     ))
    // }

    // pub fn pixels<R: RangeBounds<usize>>(&self, range: R) -> Result<PixelIter, PixelError> {
    //     let index = match range.start_bound() {
    //         Bound::Unbounded => 0,
    //         Bound::Included(i) => *i,
    //         Bound::Excluded(j) => j + 1,
    //     };

    //     let end = match range.end_bound() {
    //         Bound::Unbounded => None,
    //         Bound::Included(i) => Some(i + 1),
    //         Bound::Excluded(j) => Some(*j),
    //     };
    //     self._pixels(index, end)
    // }

    // fn _pixels(&self, index: usize, end: Option<usize>) -> Result<PixelIter, PixelError> {
    //     use TextureFormat::*;
    //     let format = self.texture_descriptor.format;
    //     let components = format.components() as usize;
    //     match format {
    //         Rgba32Float => {
    //             let f = align_to::<u8, f32>(&self.data)?;
    //             Ok(f.len() / components)
    //         }
    //         Rgba8Unorm => Ok(self.data.len() / components),
    //         Rgba8UnormSrgb => Ok(self.data.len() / components),
    //         f => {
    //             if f.is_compressed() {
    //                 Err(PixelError::CompressionNotSupported)
    //             } else if f.has_depth_aspect() {
    //                 Err(PixelError::DepthNotSupported)
    //             } else if f.has_stencil_aspect() {
    //                 Err(PixelError::StencilNotSupported)
    //             } else {
    //                 todo!("Fix {f:?}");
    //             }
    //         }
    //     }
    //     .and_then(|max_length| {
    //         let end = end.unwrap_or(max_length);
    //         if index >= max_length || end > max_length {
    //             Err(PixelError::InvalidRange)
    //         } else {
    //             Ok(PixelIter {
    //                 image: self,
    //                 index,
    //                 end,
    //             })
    //         }
    //     })
    // }
}

fn align_to<T, U>(slice: &[T]) -> Result<&[U], PixelError> {
    let (prefix, aligned, suffix) = unsafe { slice.align_to::<U>() };
    if !prefix.is_empty() || !suffix.is_empty() {
        Err(PixelError::AlignmentFailed)
    } else {
        Ok(aligned)
    }
}

fn align_to_mut<T, U>(slice: &mut [T]) -> Result<&mut [U], PixelError> {
    let (prefix, aligned, suffix) = unsafe { slice.align_to_mut::<U>() };
    if !prefix.is_empty() || !suffix.is_empty() {
        Err(PixelError::AlignmentFailed)
    } else {
        Ok(aligned)
    }
}

// #[derive(Debug)]
// pub struct PixelIter<'a> {
//     image: &'a Image,
//     index: usize,
//     end: usize,
// }

// impl<'a> Iterator for PixelIter<'a> {
//     type Item = Color;
//     fn next(&mut self) -> Option<Self::Item> {
//         use TextureFormat::*;
//         if self.index >= self.end {
//             None
//         } else {
//             let format = self.image.texture_descriptor.format;
//             let components = format.components() as usize;
//             let start = self.index * components;
//             self.index += 1;
//             match format {
//                 Rgba32Float => {
//                     let floats = align_to::<u8, f32>(&self.image.data).ok()?;
//                     let mut a = [0.0f32; 4];
//                     a.copy_from_slice(&floats[start..start + components]);
//                     Some(LinearRgba::from_f32_array(a).into())
//                 }
//                 Rgba8Unorm => {
//                     let mut a = [0u8; 4];
//                     a.copy_from_slice(&self.image.data[start..start + components]);
//                     Some(LinearRgba::from_u8_array(a).into())
//                 }
//                 Rgba8UnormSrgb => {
//                     let mut a = [0u8; 4];
//                     a.copy_from_slice(&self.image.data[start..start + components]);
//                     Some(Srgba::from_u8_array(a).into())
//                 }
//                 _ => None,
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use bevy::render::{render_asset::RenderAssetUsages, render_resource::TextureDimension};

//     #[test]
//     fn test_rgba8unorm_size() {
//         let format = TextureFormat::Rgba8Unorm;
//         assert_eq!(format.pixel_size(), 4);
//         assert_eq!(format.components(), 4);
//     }

//     #[test]
//     fn test_r8unorm_size() {
//         let format = TextureFormat::R8Unorm;
//         assert_eq!(format.pixel_size(), 1);
//         assert_eq!(format.components(), 1);
//     }

//     #[test]
//     fn test_get_pixel() {
//         let image = Image::default();
//         assert_eq!(image.texture_descriptor.size.width, 1);
//         assert_eq!(image.texture_descriptor.size.height, 1);
//         // assert_eq!(image.get_pixel(0, 0).unwrap(), Srgba::from(Color::WHITE));
//         assert_eq!(
//             image.get_pixel(UVec2::new(0, 0)).unwrap(),
//             Srgba::WHITE.into()
//         );
//         assert_eq!(
//             image.get_pixel(UVec2::new(1, 0)).unwrap_err(),
//             PixelError::InvalidLocation
//         );
//         assert_eq!(
//             image.get_pixel(UVec2::new(0, 1)).unwrap_err(),
//             PixelError::InvalidLocation
//         );
//     }

//     #[test]
//     fn test_align_to_from_f32() {
//         let pixel = [0.0, 0.0, 0.0, 1.0];
//         assert!(&align_to::<f32, u8>(&pixel).is_ok());

//         // We can always go to u8 from f32.
//         let pixel = [0.0, 0.0, 0.0, 1.0, 0.0];
//         assert!(&align_to::<f32, u8>(&pixel).is_ok());

//         // We can always go to u8 from f32.
//         let pixel = [0.0, 0.0, 0.0];
//         assert!(&align_to::<f32, u8>(&pixel).is_ok());
//     }

//     #[test]
//     fn test_align_to_f32_from_u8() {
//         let pixel = [0u8; 4];

//         // let (prefix, aligned, suffix) = unsafe { pixel.align_to::<f32>() };
//         // assert!(prefix.is_empty());
//         // assert!(suffix.is_empty());
//         // assert_eq!(aligned.len(), 1);

//         assert!(align_to::<u8, f32>(&pixel).is_ok());

//         let pixel = [0u8; 17];
//         assert!(align_to::<u8, f32>(&pixel).is_err());

//         let pixel = [0u8; 15];
//         assert!(align_to::<u8, f32>(&pixel).is_err());
//     }

//     fn image_from<T>(
//         width: u32,
//         height: u32,
//         format: TextureFormat,
//         data: &[T],
//     ) -> Result<Image, PixelError> {
//         let size = Extent3d {
//             width,
//             height,
//             depth_or_array_layers: 1,
//         };
//         Ok(Image::new_fill(
//             size,
//             TextureDimension::D2,
//             align_to::<T, u8>(data)?,
//             format,
//             RenderAssetUsages::MAIN_WORLD,
//         ))
//     }

//     #[test]
//     fn test_get_pixel_f32() {
//         let pixel = [0.0, 0.0, 0.0, 1.0];
//         // FIXME: Spooky. If the next line is removed, the following image_from() will fail.
//         // Must have to do with alignment.
//         assert_eq!(align_to::<f32, u8>(&pixel).unwrap().len(), 16);
//         let image = image_from(1, 1, TextureFormat::Rgba32Float, &pixel).unwrap();
//         assert_eq!(
//             image.get_pixel(UVec2::new(0, 0)).unwrap(),
//             LinearRgba::BLACK.into()
//         );
//     }

//     #[test]
//     fn test_pixels() {
//         let pixel = [0.0, 0.0, 0.0, 1.0];
//         // FIXME: Spooky. If the next line is removed, the following image_from() will fail.
//         // Must have to do with alignment.
//         assert_eq!(align_to::<f32, u8>(&pixel).unwrap().len(), 16);
//         let image = image_from(1, 1, TextureFormat::Rgba32Float, &pixel).unwrap();
//         let mut pixels = image.pixels(..).unwrap();
//         assert_eq!(pixels.next().unwrap(), LinearRgba::BLACK.into());
//         assert_eq!(pixels.next(), None);

//         assert_eq!(image.pixels(1..).unwrap_err(), PixelError::InvalidRange);
//         assert_eq!(image.pixels(..=1).unwrap_err(), PixelError::InvalidRange);
//         assert!(image.pixels(0..0).unwrap().next().is_none());
//         assert!(image.pixels(0..1).unwrap().next().is_some());
//     }

//     // fn get_rgba(image: &Image, loc: impl Into<PixelLoc>) -> LinearRgba {
//     //     LinearRgba::from(image.get_pixel(loc).unwrap())
//     // }

//     #[test]
//     fn test_r8image() {
//         let pixel = [255u8];
//         // FIXME: Spooky. If the next line is removed, the following
//         // image_from() will fail. Must have to do with alignment.
//         let image = image_from(1, 1, TextureFormat::R8Unorm, &pixel).unwrap();
//         assert_eq!(image.get_pixel(0).unwrap(), LinearRgba::RED.into());

//         let image = image_from(1, 1, TextureFormat::R8Uint, &pixel).unwrap();
//         assert_eq!(get_rgba(&image, 0).red, u8::MAX as f32);

//         let pixel = [127i8];
//         let image = image_from(1, 1, TextureFormat::R8Snorm, &pixel).unwrap();
//         assert_eq!(image.get_pixel(0).unwrap(), LinearRgba::RED.into());

//         let image = image_from(1, 1, TextureFormat::R8Sint, &pixel).unwrap();
//         assert_eq!(get_rgba(&image, 0).red, i8::MAX as f32);
//     }

//     #[test]
//     fn from_pixels() {
//         let image = Image::from_pixels(&[LinearRgba::RED], 1).unwrap();
//         assert_eq!(image.get_pixel(0).unwrap(), LinearRgba::RED.into());
//     }
// }
