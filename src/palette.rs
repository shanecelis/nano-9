
use bevy::{
    ecs::system::SystemState,
    prelude::*,
};

use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::{
    DrawState,
    assets::{ImageHandles},
    pixel::PixelAccess,
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
        let mut system_state: SystemState<(Res<Nano9Palette>, Res<Assets<Image>>, Res<DrawState>)> =
            SystemState::new(world);
        let (palette, images, draw_state) = system_state.get(world);
        match c {
            Value::Integer(n) => {
                images.get(&palette.0)
                      .and_then(|pal| pal.get_pixel(n as usize).ok()).unwrap_or(draw_state.pen)
            }
            _ => draw_state.pen
        }
    }
}
