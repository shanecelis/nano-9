use std::sync::Mutex;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::system::SystemState,
    prelude::*,
    reflect::Reflect,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    utils::Duration,
    window::PresentMode,
    window::{PrimaryWindow, WindowResized, WindowResolution},
};

use bevy_asset_loader::prelude::*;
use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    MetaMethod, UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::{
    assets::{self, ImageHandles},
    pixel::PixelAccess,
    screens,
};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);

}

fn setup(mut commands: Commands,
    image_handles: Res<ImageHandles>,
) {
    commands.insert_resource(Nano9Palette(
        image_handles
            .get(ImageHandles::PICO8_PALETTE)
            .unwrap()
            .clone(),
    ));
}

#[derive(Resource)]
pub struct Nano9Palette(pub Handle<Image>);

impl Nano9Palette {
    pub fn get_color(c: Value, world: &mut World) -> Color {
        let mut system_state: SystemState<(Res<Nano9Palette>, Res<Assets<Image>>)> =
            SystemState::new(world);
        let (palette, images) = system_state.get(&world);
        match c {
            Value::Integer(n) => {
                let pal = images.get(&palette.0).unwrap();
                pal.get_pixel(n as usize).unwrap()
            }
            _ => todo!(),
        }
    }
}
