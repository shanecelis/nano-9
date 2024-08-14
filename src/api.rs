#![allow(deprecated)]
use std::sync::Mutex;

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
    Nano9Palette,
    Nano9Screen,
    Nano9SpriteSheet,
    MySprite,
    pixel::PixelAccess,
};


#[derive(Clone)]
pub struct MyHandle<T: Asset + Clone>(pub Handle<T>);

impl<T: Asset + Clone> UserData for MyHandle<T> {}
// We can implement `FromLua` trait for our `Vec2` to return a copy
impl<T: Asset + Clone> FromLua<'_> for MyHandle<T> {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

// impl<'lua, T: Asset> IntoLua<'lua> for MyHandle<T> {
//     fn into_lua(self, lua: &'lua Lua) -> mlua::Result<Value<'lua>> {
//         Ok(Value::UserData(self.into()))
//     }
// }


pub fn plugin(app: &mut App) {
    app
        .add_systems(FixedUpdate, script_event_handler::<LuaScriptHost<()>, 0, 1>)
        // .register_foreign_lua_type::<Handle<Image>>()
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
                "spr",
                // ctx.create_function(|ctx, (n, x, y): (usize, f32, f32)| {
                ctx.create_function(|ctx, n: i32| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<Res<Nano9SpriteSheet>> =
                        SystemState::new(&mut world);
                    let sprite_sheet = system_state.get(&world);

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
                    let image = images.get_mut(&screen.0).unwrap();
                    let _ = image.set_pixels(|_, _| c);
                    Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "loadimg",
                ctx.create_function(|ctx, path: String| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<(
                        Res<AssetServer>,
                        ResMut<Assets<Image>>,
                    )> = SystemState::new(&mut world);
                    let (server, mut images) = system_state.get_mut(&mut world);
                    let handle: Handle<Image> = server.load(&path);
                    Ok(MyHandle(handle))
                    // Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        ctx.globals()
            .set(
                "setpal",
                ctx.create_function(|ctx, img: MyHandle<Image>| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    world.insert_resource(Nano9Palette(img.0));
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
