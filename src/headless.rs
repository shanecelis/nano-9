use bevy::{
    image::ImageSampler,
    prelude::*,
    render::render_resource::{TextureFormat, TextureUsages},
};

/// Marker: GPU headless (no Winit window). Camera renders to [`CanvasRenderTarget`].
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct Headless;

/// GPU image [`Nano9Camera`](crate::pico8::Nano9Camera) draws into when [`Headless`].
#[derive(Resource, Debug, Clone)]
pub struct CanvasRenderTarget(pub Handle<Image>);

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<Headless>();
}

/// Canvas-sized render target with `COPY_SRC` for `Screenshot::image`.
pub(crate) fn setup_render_target(images: &mut Assets<Image>, size: UVec2) -> Handle<Image> {
    let mut image = Image::new_target_texture(
        size.x.max(1),
        size.y.max(1),
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
    image.sampler = ImageSampler::nearest();
    images.add(image)
}
