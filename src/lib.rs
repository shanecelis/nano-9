
use bevy::{
    asset::AssetMetaCheck,
    // audio::{AudioPlugin, Volume},
    prelude::*,
};
use bevy::ecs::schedule::SystemSet;
mod sprite;
mod palette;
mod api;
mod pixel;
mod plugin;
mod assets;
pub mod screens;
pub use plugin::*;
pub use sprite::*;
pub use palette::*;



pub(crate) fn plugin(app: &mut App) {

    // Add other plugins.
    app.add_plugins((
        // demo::plugin,
        screens::plugin,
        // theme::plugin,
        assets::plugin,
        api::plugin,
        // sprite::plugin,
        palette::plugin,
        // audio::plugin,
    ));

    // Enable dev tools for dev builds.
    // #[cfg(feature = "dev")]
    // app.add_plugins(dev_tools::plugin);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2dBundle::default(),
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
    ));
}
