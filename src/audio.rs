use bevy::{
    ecs::system::SystemState,
    audio::PlaybackMode,
    prelude::*,
};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::{
    api::MyHandle,
    MySprite,
    palette::Nano9Palette,
};

#[derive(Clone)]
pub struct N9AudioLoader;
impl FromLua<'_> for N9AudioLoader {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl UserData for N9AudioLoader {

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {

        methods.add_method_mut("load", |ctx, this, path: String| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(
                Res<AssetServer>,
            )> = SystemState::new(&mut world);
            let (server,) = system_state.get(& world);
            let handle: Handle<AudioSource> = server.load(&path);
            Ok(N9Audio { handle })
        });
    }
}

#[derive(Clone)]
pub struct N9Audio {
    pub handle: Handle<AudioSource>,
}

impl FromLua<'_> for N9Audio {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl UserData for N9Audio {

    // fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
    //     fields.add_field_method_get("x", |ctx, this| {
    //         Ok(())
    //     });
    // }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {

        methods.add_method_mut("sfx", |ctx, this, _: ()| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            world.spawn(AudioBundle {
                source: this.handle.clone(),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    ..default()
                }
            });
            Ok(())
        });
    }
}
