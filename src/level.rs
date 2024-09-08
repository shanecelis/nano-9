use bevy::prelude::*;

use bevy::{
    ecs::{
        system::SystemState,
    }
};
use bevy_ecs_ldtk::prelude::*;
use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{UserData, UserDataMethods, UserDataFields};
use bevy_mod_scripting::prelude::*;

pub(crate) fn plugin(app: &mut App) {

}

#[derive(Clone)]
pub struct N9LevelLoader;

impl FromLua<'_> for N9LevelLoader {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl UserData for N9LevelLoader {

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("load", |ctx, this, path: String| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Res<AssetServer>,)> = SystemState::new(&mut world);
            let (server,) = system_state.get(&world);
            let level: Handle<LdtkProject> = server.load(path);
            Ok(N9Level(world.spawn(level).id()))
        });
    }
}


#[derive(Clone)]
pub struct N9Level(Entity);

impl FromLua<'_> for N9Level {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl UserData for N9Level {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        // fields.add_field_method_get("default", |ctx, this| Ok(N9TextStyle::default()));
    }
}
