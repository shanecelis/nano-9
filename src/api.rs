#![allow(deprecated)]
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
    Nano9Palette,
    Nano9Screen,
    Nano9SpriteSheet,
    MySprite,
    assets::{self, ImageHandles},
    pixel::PixelAccess,
    screens,
};

pub fn plugin(app: &mut App) {
    app
        .add_systems(FixedUpdate, script_event_handler::<LuaScriptHost<()>, 0, 1>)
        .add_script_host::<LuaScriptHost<()>>(PostUpdate)
        .add_api_provider::<LuaScriptHost<()>>(Box::new(LuaCoreBevyAPIProvider))
        .add_api_provider::<LuaScriptHost<()>>(Box::new(Nano9API));
}


#[derive(Default)]
pub struct Nano9API;

impl APIProvider for Nano9API {
    type APITarget = Mutex<Lua>;
    type ScriptContext = Mutex<Lua>;
    type DocTarget = LuaDocFragment;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        // callbacks can receive any `ToLuaMulti` arguments, here '()' and
        // return any `FromLuaMulti` arguments, here a `usize`
        // check the Rlua documentation for more details

        let ctx = ctx.get_mut().unwrap();

        ctx.globals()
            .set(
                "pset",
                ctx.create_function(|ctx, (x, y, c): (f32, f32, Value)| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<(
                        Res<Nano9Screen>,
                        Res<Nano9Palette>,
                        ResMut<Assets<Image>>,
                    )> = SystemState::new(&mut world);
                    let (screen, palette, mut images) = system_state.get_mut(&mut world);
                    let color = match c {
                        Value::Integer(n) => {
                            let pal = images.get(&palette.0).unwrap();
                            pal.get_pixel(n as usize).unwrap()
                        }
                        _ => todo!(),
                    };
                    let mut image = images.get_mut(&screen.0).unwrap();
                    let _ = image.set_pixel((x as usize, y as usize), color);
                    Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "time",
                ctx.create_function(|ctx, _: ()| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<(Res<Time>)> = SystemState::new(&mut world);
                    let (time) = system_state.get(&world);
                    Ok(time.elapsed_seconds())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;
        ctx.globals()
            .set(
                "btn",
                ctx.create_function(|ctx, (b): (u8)| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<(Res<ButtonInput<KeyCode>>)> =
                        SystemState::new(&mut world);
                    let (input) = system_state.get(&world);
                    Ok(input.pressed(match b {
                        0 => KeyCode::ArrowLeft,
                        1 => KeyCode::ArrowRight,
                        2 => KeyCode::ArrowUp,
                        3 => KeyCode::ArrowDown,
                        4 => KeyCode::KeyZ,
                        5 => KeyCode::KeyX,
                        x => todo!("key {x:?}"),
                    }))
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;
        ctx.globals()
            .set(
                "btnp",
                ctx.create_function(|ctx, (b): (u8)| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<(Res<ButtonInput<KeyCode>>)> =
                        SystemState::new(&mut world);
                    let (input) = system_state.get(&world);
                    Ok(input.just_pressed(match b {
                        0 => KeyCode::ArrowLeft,
                        1 => KeyCode::ArrowRight,
                        2 => KeyCode::ArrowUp,
                        3 => KeyCode::ArrowDown,
                        4 => KeyCode::KeyZ,
                        5 => KeyCode::KeyX,
                        x => todo!("key {x:?}"),
                    }))
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;
        ctx.globals()
            .set(
                "spr",
                // ctx.create_function(|ctx, (n, x, y): (usize, f32, f32)| {
                ctx.create_function(|ctx, (n): (i32)| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<(Res<Nano9SpriteSheet>)> =
                        SystemState::new(&mut world);
                    let (sprite_sheet) = system_state.get(&world);

                    let bundle = (
                        SpriteBundle {
                            texture: sprite_sheet.0.clone(),
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(24.0, 24.0)),
                                ..default()
                            },
                            ..default()
                        },
                        TextureAtlas {
                            layout: sprite_sheet.1.clone(),
                            index: n as usize,
                        },
                    );
                    Ok(MySprite(world.spawn(bundle).id()))
                    // Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "cls",
                ctx.create_function(|ctx, value| {

                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let c = Nano9Palette::get_color(value, &mut world);
                    let mut system_state: SystemState<(
                        Res<Nano9Screen>,
                        ResMut<Assets<Image>>,
                    )> = SystemState::new(&mut world);
                    let (screen, mut images) = system_state.get_mut(&mut world);
                    let mut image = images.get_mut(&screen.0).unwrap();
                    let _ = image.set_pixels(|_, _| c);
                    Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        Ok(())
    }

    fn register_with_app(&self, app: &mut App) {
        // app.register_type::<Settings>();
    }
}
