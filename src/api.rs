#![allow(deprecated)]
use std::sync::{Arc, Mutex};

use bevy::{
    ecs::system::SystemState,
    prelude::*,
    reflect::Reflect,
};
use bevy_mod_scripting::{
    prelude::*,
    api::lua::RegisterForeignLuaType,
};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{self,
    UserData, UserDataFields, UserDataMethods,
};
// use bevy_pixel_buffer::prelude::*;
use crate::{
    DrawState,
    N9Camera,
    N9Error,
    N9Sprite,
    N9Image,
    N9TextLoader,
    N9TextStyle,
    N9ImageLoader,
    N9AudioLoader,
    Nano9Palette,
    Nano9Screen,
    Nano9SpriteSheet,
    pixel::PixelAccess,
};


#[derive(Clone)]
pub struct MyHandle<T: Asset + Clone>(pub Handle<T>);

// We can implement `FromLua` trait for our `Vec2` to return a copy
impl<T: Asset + Clone> FromLua<'_> for MyHandle<T> {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl<T: Asset + Clone> UserData for MyHandle<T> {}

// type N9Arg = ();

#[derive(Clone, Default)]
pub enum N9Arg {
    #[default]
    None,
    ImagePair { name: String, image: N9Image },
    SetCamera { name: String, camera: Entity },
    SetSprite { name: String, sprite: Entity }
}

impl UserData for N9Arg { }

impl FromLua<'_> for N9Arg {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

pub fn plugin(app: &mut App) {
    app
        .add_plugins(ScriptingPlugin)
        .add_systems(FixedUpdate, script_event_handler::<LuaScriptHost<N9Arg>, 0, 1>)
        // .register_foreign_lua_type::<Handle<Image>>()
        .add_script_host::<LuaScriptHost<N9Arg>>(PostUpdate)
        .add_api_provider::<LuaScriptHost<N9Arg>>(Box::new(LuaCoreBevyAPIProvider))
        .add_api_provider::<LuaScriptHost<N9Arg>>(Box::new(Nano9API))
        .add_script_handler::<LuaScriptHost<N9Arg>, 0, 0>(PostUpdate);
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
                "audio",
                N9AudioLoader,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "image",
                N9ImageLoader,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "text",
                N9TextLoader,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "_set_global",
                ctx.create_function(|ctx, (arg): (N9Arg)| {
                    match arg {
                        N9Arg::ImagePair { name, image } => {
                            warn!("set global {name}");
                            ctx.globals().set(
                                name,
                                image)
                        }
                        N9Arg::SetSprite { name, sprite } => {
                            ctx.globals().set(
                                name,
                                N9Sprite(sprite))
                        }
                        N9Arg::SetCamera { name, camera } => {
                            ctx.globals().set(
                                name,
                                N9Camera(camera))
                        }
                        _ => {
                            // XXX: This should be an error.
                            Ok(())
                        }
                    }
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "pset",
                ctx.create_function(|ctx, (x, y, c): (f32, f32, Value)| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let color = Nano9Palette::get_color(c, &mut world);
                    let mut system_state: SystemState<(
                        Res<Nano9Screen>,
                        ResMut<Assets<Image>>,
                    )> = SystemState::new(&mut world);
                    let (screen, mut images) = system_state.get_mut(&mut world);
                    let image = images.get_mut(&screen.0).unwrap();
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
                    let mut system_state: SystemState<Res<Time>> = SystemState::new(&mut world);
                    let time = system_state.get(&world);
                    Ok(time.elapsed_seconds())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "delta_time",
                ctx.create_function(|ctx, _: ()| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<Res<Time>> = SystemState::new(&mut world);
                    let time = system_state.get(&world);
                    Ok(time.delta_seconds())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;
        ctx.globals()
            .set(
                "btn",
                ctx.create_function(|ctx, b: u8| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<Res<ButtonInput<KeyCode>>> =
                        SystemState::new(&mut world);
                    let input = system_state.get(&world);
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
                ctx.create_function(|ctx, b: u8| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<Res<ButtonInput<KeyCode>>> =
                        SystemState::new(&mut world);
                    let input = system_state.get(&world);
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
                    let image = images.get_mut(&screen.0).unwrap();
                    let _ = image.set_pixels(|_, _| c);
                    Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "setpal",
                ctx.create_function(|ctx, img: N9Image| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    world.insert_resource(Nano9Palette(img.handle.clone()));
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
