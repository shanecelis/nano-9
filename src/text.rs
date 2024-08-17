use bevy::{
    ecs::system::SystemState,
    audio::PlaybackMode,
    prelude::*,
};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;

#[derive(Clone)]
pub struct N9TextLoader;
impl FromLua<'_> for N9TextLoader {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl UserData for N9TextLoader {

    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("default", |ctx, this| {
            Ok(N9TextStyle::default())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {

        methods.add_method_mut("load", |ctx, this, path: String| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(
                Res<AssetServer>,
            )> = SystemState::new(&mut world);
            let (server,) = system_state.get(& world);
            let font: Handle<Font> = server.load(&path);
            Ok(N9TextStyle(TextStyle { font, ..default() }))
        });
    }
}

#[derive(Clone, Default)]

pub struct N9TextStyle(TextStyle);
impl FromLua<'_> for N9TextStyle {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl UserData for N9TextStyle {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("print", |ctx, this, str: String| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let id = world.spawn(Text2dBundle {
                text: Text::from_section(str, this.0.clone()),
                ..default()
            }).id();
            Ok(())
        });
    }
}
