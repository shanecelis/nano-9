use bevy::{
    ecs::{
        system::SystemState,
        world::{Command, CommandQueue},
    },
    prelude::*,
};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;

use std::sync::OnceLock;

const RESERVE_ENTITY_COUNT: usize = 10;

pub(crate) fn reserved_entities() -> Option<&'static mut Vec<Entity>> {
    static mut MEM: OnceLock<Vec<Entity>> = OnceLock::new();
    unsafe {
        let _ = MEM.get_or_init(Vec::new);
        MEM.get_mut()
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, reserve_entities);
    app.add_systems(PreUpdate, reserve_entities);
    app.add_systems(PostUpdate, run_deferred_commands);
}

pub fn reserve_entities(world: &mut World) {
    let Some(entities) = reserved_entities() else {
        return;
    };
    let delta = RESERVE_ENTITY_COUNT - entities.len();
    if delta > 0 {
        for e in world.entities().reserve_entities(delta as u32) {
            entities.push(e);
        }
    }
}

pub fn run_deferred_commands(world: &mut World) {
    let Some(commands) = deferred_commands() else {
        return;
    };
    commands.apply(world);
}

pub(crate) fn deferred_commands() -> Option<&'static mut CommandQueue> {
    static mut MEM: OnceLock<CommandQueue> = OnceLock::new();
    unsafe {
        let _ = MEM.get_or_init(CommandQueue::default);
        MEM.get_mut()
    }
}

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

struct Print(Entity, String, TextStyle);

impl Command for Print {
    fn apply(self, world: &mut World) {
        world.entity_mut(self.0).insert(Text2dBundle {
            text: Text::from_section(self.1, self.2),
            ..default()
        });
    }
}

impl UserData for N9TextLoader {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("default", |ctx, this| Ok(N9TextStyle::default()));
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("load", |ctx, this, path: String| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Res<AssetServer>,)> = SystemState::new(&mut world);
            let (server,) = system_state.get(&world);
            let font: Handle<Font> = server.load(path);
            Ok(N9TextStyle(TextStyle { font, ..default() }))
        });

        methods.add_method_mut("print", |ctx, this, str: String| {
            if let Ok(world) = ctx.get_world() {
                let mut world = world.write();
                let id = world
                    .spawn(Text2dBundle {
                        text: Text::from_section(str, TextStyle::default()),
                        ..default()
                    })
                    .id();
            } else if let Some(entities) = reserved_entities() {
                if let Some(id) = entities.pop() {
                    if let Some(c) = deferred_commands() {
                        c.push(Print(id, str, TextStyle::default()))
                    }
                } else {
                    warn!("Ran out of reserved entities.");
                }
            }
            Ok(())
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
            let id = world
                .spawn(Text2dBundle {
                    text: Text::from_section(str, this.0.clone()),
                    ..default()
                })
                .id();
            Ok(())
        });
    }
}
