use super::raster::Raster;
use super::*;
use crate::translate::Position;

pub(crate) fn plugin(app: &mut App) {
    #[cfg(feature = "scripting")]
    lua::plugin(app);
}

bobtail::define! {
    #[doc(hidden)]
    pub __oval => fn oval(
        &mut self,
        upper_left: Vec2,
        lower_right: Vec2,
        #[tail]
        color: Option<PColor>,
    ) -> Result<Entity, Error>;
    #[doc(hidden)]
    pub __ovalfill => fn ovalfill(
        &mut self,
        upper_left: Vec2,
        lower_right: Vec2,
        #[tail]
        color: Option<PColor>,
    ) -> Result<Entity, Error>;
}
pub use __oval as oval;
pub use __ovalfill as ovalfill;

fn spawn_oval(
    pico8: &mut super::Pico8<'_, '_>,
    name: &'static str,
    upper_left: Vec2,
    lower_right: Vec2,
    color: Color,
    fill: bool,
) -> Result<Entity, Error> {
    let pen = Srgba::from(color).to_u8_array();
    let mut x0 = upper_left.x.floor() as i32;
    let mut y0 = upper_left.y.floor() as i32;
    let mut x1 = lower_right.x.floor() as i32;
    let mut y1 = lower_right.y.floor() as i32;
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }
    let size = UVec2::new((x1 - x0 + 1) as u32, (y1 - y0 + 1) as u32);
    let mut raster = Raster::new(size, pen);
    // Raster in image-local coordinates so (x0, y0) is the top-left pixel.
    if fill {
        raster.ovalfill(0, 0, x1 - x0, y1 - y0);
    } else {
        raster.oval(0, 0, x1 - x0, y1 - y0);
    }
    let handle = pico8.images.add(raster.image);
    let clearable = Clearable::default();
    let id = pico8
        .commands
        .spawn((
            Name::new(name),
            Sprite {
                image: handle,
                custom_size: Some(size.as_vec2()),
                ..default()
            },
            Anchor::TOP_LEFT,
            Position::from(Vec2::new(x0 as f32, y0 as f32)),
            clearable,
        ))
        .id();
    pico8.state.draw_state.mark_drawn();
    Ok(id)
}

impl super::Pico8<'_, '_> {
    pub fn ovalfill(
        &mut self,
        upper_left: Vec2,
        lower_right: Vec2,
        color: Option<PColor>,
    ) -> Result<Entity, Error> {
        let color = self.get_color(color)?;
        spawn_oval(self, "ovalfill", upper_left, lower_right, color, true)
    }

    pub fn oval(
        &mut self,
        upper_left: Vec2,
        lower_right: Vec2,
        color: Option<PColor>,
    ) -> Result<Entity, Error> {
        let color = self.get_color(color)?;
        spawn_oval(self, "oval", upper_left, lower_right, color, false)
    }
}

#[cfg(feature = "scripting")]
mod lua {
    use super::*;
    use crate::pico8::lua::with_pico8;

    use bevy_mod_scripting::bindings::function::{
        namespace::{GlobalNamespace, NamespaceBuilder},
        script_function::FunctionCallContext,
    };
    pub(crate) fn plugin(app: &mut App) {
        let world = app.world_mut();

        NamespaceBuilder::<GlobalNamespace>::new_unregistered(world)
            .register(
                "ovalfill",
                |ctx: FunctionCallContext,
                 x0: Option<f32>,
                 y0: Option<f32>,
                 x1: Option<f32>,
                 y1: Option<f32>,
                 c: Option<PColor>| {
                    let _ = with_pico8(&ctx, move |pico8| {
                        pico8.ovalfill(
                            Vec2::new(x0.unwrap_or(0.0), y0.unwrap_or(0.0)),
                            Vec2::new(x1.unwrap_or(0.0), y1.unwrap_or(0.0)),
                            c,
                        )
                    })?;
                    Ok(())
                },
            )
            .register(
                "oval",
                |ctx: FunctionCallContext,
                 x0: Option<f32>,
                 y0: Option<f32>,
                 x1: Option<f32>,
                 y1: Option<f32>,
                 c: Option<PColor>| {
                    let _ = with_pico8(&ctx, move |pico8| {
                        pico8.oval(
                            Vec2::new(x0.unwrap_or(0.0), y0.unwrap_or(0.0)),
                            Vec2::new(x1.unwrap_or(0.0), y1.unwrap_or(0.0)),
                            c,
                        )
                    })?;
                    Ok(())
                },
            );
    }
}
