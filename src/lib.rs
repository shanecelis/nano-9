
use bevy::{
    prelude::*,
};
mod sprite;
mod palette;
pub mod api;
mod pixel;
mod plugin;
mod assets;
mod image;
mod audio;
mod text;
mod error;
mod camera;
pub mod screens;

pub use plugin::*;
pub use sprite::*;
pub use image::*;
pub use palette::*;
pub use audio::*;
pub use text::*;
pub use camera::*;

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

