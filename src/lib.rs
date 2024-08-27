use bevy::prelude::*;
pub mod api;
mod assets;
mod audio;
mod camera;
mod error;
mod image;
mod palette;
mod pixel;
mod plugin;
pub mod screens;
mod sprite;
mod text;
mod color;
mod ext;

pub use audio::*;
pub use camera::*;
pub use image::*;
pub use palette::*;
pub use plugin::*;
pub use sprite::*;
pub use text::*;
pub use ext::*;
pub use color::*;

#[derive(thiserror::Error, Debug)]
pub enum N9Error {
    #[error("palette unavailable")]
    PaletteUnavailable,
}

pub(crate) fn plugin(app: &mut App) {
    // Add other plugins.
    app.add_plugins((
        // demo::plugin,
        screens::plugin,
        // theme::plugin,
        assets::plugin,
        api::plugin,
        sprite::plugin,
        palette::plugin,
        error::plugin,
        text::plugin,
        // audio::plugin,
    ));

    // Enable dev tools for dev builds.
    // #[cfg(feature = "dev")]
    // app.add_plugins(dev_tools::plugin);
}
