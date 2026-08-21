use bevy::{
    image::ImageSampler,
    prelude::*,
    render::render_resource::{TextureFormat, TextureUsages},
};

/// Marker: GPU headless (no Winit window). Camera renders to [`CanvasRenderTarget`].
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Headless;

/// Typical window DPI. `Text2d` rasterizes at `font_size * scale_factor` and
/// extract then scales glyphs by `1 / scale_factor`. A 128×128 image at 1.0
/// therefore minifies 10px glyphs into 5px and the first glyph's gap fills in.
/// Match a Retina window: physical size × this factor, same `scale_factor` on
/// the image target **and** on `Screenshot` (`Screenshot::image` always uses
/// 1.0, which does not match and captures black).
pub(crate) const TEXT2D_LAYOUT_SCALE: f32 = 2.0;

/// GPU image [`Nano9Camera`](crate::pico8::Nano9Camera) draws into when [`Headless`].
#[derive(Resource, Debug, Clone)]
pub struct CanvasRenderTarget {
    pub handle: Handle<Image>,
    pub scale_factor: f32,
}

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Headless>();
}

/// Physical-pixel render target with `COPY_SRC` for screenshots.
pub(crate) fn setup_render_target(
    images: &mut Assets<Image>,
    physical_size: UVec2,
) -> Handle<Image> {
    let mut image = Image::new_target_texture(
        physical_size.x.max(1),
        physical_size.y.max(1),
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    image.sampler = ImageSampler::nearest();
    images.add(image)
}
