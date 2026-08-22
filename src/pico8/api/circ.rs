use super::raster::Raster;
use super::*;
use crate::translate::Position;

pub(crate) fn plugin(app: &mut App) {
    #[cfg(feature = "scripting")]
    lua::plugin(app);
}

bobtail::define! {
    #[doc(hidden)]
    pub __circ => fn circ(
        &mut self,
        pos: Vec2,
        r: impl Into<UVec2>,
        #[tail]
        color: Option<PColor>,
    ) -> Result<Entity, Error>;
    #[doc(hidden)]
    pub __circfill => fn circfill(
        &mut self,
        pos: Vec2,
        r: impl Into<UVec2>,
        #[tail]
        color: Option<PColor>,
    ) -> Result<Entity, Error>;
}
pub use __circ as circ;
pub use __circfill as circfill;

fn spawn_circ(
    pico8: &mut super::Pico8<'_, '_>,
    name: &'static str,
    pos: Vec2,
    r: UVec2,
    color: Color,
    fill: bool,
) -> Result<Entity, Error> {
    let pen = Srgba::from(color).to_u8_array();
    let radius = r.x as i32;
    let size = UVec2::splat(r.x.saturating_mul(2).saturating_add(1));
    let mut raster = Raster::new(size, pen);
    if fill {
        raster.circfill(radius, radius, radius);
    } else {
        raster.circ(radius, radius, radius);
    }
    let handle = pico8.images.add(raster.image);
    let origin = pos - Vec2::splat(r.x as f32);
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
            Position::from(origin),
            clearable,
        ))
        .id();
    pico8.state.draw_state.mark_drawn();
    Ok(id)
}

impl super::Pico8<'_, '_> {
    pub fn circfill(
        &mut self,
        pos: Vec2,
        r: impl Into<UVec2>,
        color: Option<PColor>,
    ) -> Result<Entity, Error> {
        let color = self.get_color(color)?;
        spawn_circ(self, "circfill", pos, r.into(), color, true)
    }

    pub fn circ(
        &mut self,
        pos: Vec2,
        r: impl Into<UVec2>,
        color: Option<PColor>,
    ) -> Result<Entity, Error> {
        let color = self.get_color(color)?;
        spawn_circ(self, "circ", pos, r.into(), color, false)
    }
}

#[cfg(feature = "scripting")]
mod lua {
    use super::*;
    use crate::{pico8::lua::with_pico8, DropPolicy, N9Entity};

    use bevy_mod_scripting::bindings::{
        function::{
            namespace::{GlobalNamespace, NamespaceBuilder},
            script_function::FunctionCallContext,
        },
        InteropError, ScriptValue,
    };
    pub(crate) fn plugin(app: &mut App) {
        let world = app.world_mut();

        NamespaceBuilder::<GlobalNamespace>::new_unregistered(world)
            .register(
                "circfill",
                |ctx: FunctionCallContext,
                 x0: Option<f32>,
                 y0: Option<f32>,
                 r: Option<u32>,
                 c: Option<PColor>|
                 -> Result<ScriptValue, InteropError> {
                    let id = with_pico8(&ctx, move |pico8| {
                        pico8.circfill(
                            Vec2::new(x0.unwrap_or(0.0), y0.unwrap_or(0.0)),
                            UVec2::splat(r.unwrap_or(4)),
                            c,
                        )
                    })?;

                    let entity = N9Entity {
                        entity: id,
                        drop: DropPolicy::Nothing,
                    };
                    let world = ctx.world()?;
                    entity.into_script_ref(world)
                },
            )
            .register(
                "circ",
                |ctx: FunctionCallContext,
                 x0: Option<f32>,
                 y0: Option<f32>,
                 r: Option<u32>,
                 c: Option<PColor>| {
                    let _ = with_pico8(&ctx, move |pico8| {
                        pico8.circ(
                            Vec2::new(x0.unwrap_or(0.0), y0.unwrap_or(0.0)),
                            UVec2::splat(r.unwrap_or(4)),
                            c,
                        )
                    })?;
                    Ok(())
                },
            );
    }
}
