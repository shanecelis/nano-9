use bevy::prelude::*;

use bevy::{
    ecs::{
        system::SystemState,
    }
};
use bevy_ecs_ldtk::prelude::*;
use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{UserData, UserDataMethods, UserDataFields, Function, Table};
use bevy_mod_scripting::prelude::*;
use crate::{DropPolicy, EntityRep, UserDataComponent, api::{N9Arg, N9Args}};
use std::collections::HashMap;

pub(crate) fn plugin(app: &mut App) {
    app
        .add_plugins(LdtkPlugin)
        // .register_ldtk_entity::<Slime>("Slime")
        .insert_resource(LevelSelection::index(0))
        .add_systems(PostUpdate, process_entities);
}

#[derive(Clone)]
pub struct N9LevelLoader;

#[derive( Default, Bundle, LdtkEntity)]
pub struct Slime { }

#[derive(Clone, Component)]
pub struct N9LevelProcessor(HashMap<String, String>);

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
        methods.add_method_mut("load", |ctx, this, (path, table): (String, Option<Table>)| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Res<AssetServer>,)> = SystemState::new(&mut world);
            let (server,) = system_state.get(&world);
            let level: Handle<LdtkProject> = server.load(path);
            let mut l = world.spawn((LdtkWorldBundle {
                ldtk_handle: level,
                ..default()
            },
            Name::new("level")));
            if let Some(table) = table {
                let mut map = HashMap::new();
                for pair in table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    map.insert(key, value);
                }
                info!("Add table {:?}", &map);
                l.insert(N9LevelProcessor(map));
            }
            Ok(N9Level(l.id()))
        });
    }
}

fn process_entities(
    mut commands: Commands,
    new_entity_instances: Query<(Entity, &EntityInstance), Added<EntityInstance>>,
    // processor: Query<&N9LevelProcessor>,
    assets: Res<AssetServer>,
    mut events: PriorityEventWriter<LuaEvent<N9Args>>,
)
{
    // let Ok(processor) = processor.get_single() else { return; };

    // info!("process entities");
    for (entity, entity_instance) in new_entity_instances.iter() {
        events.send(
            LuaEvent {
                hook_name: "_ldtk_entity".to_owned(),
                args: {
                    let mut args = N9Args::new();
                    args.push(N9Arg::String(entity_instance.identifier.clone()));
                    args.push(N9Arg::Entity(entity));
                    args
                },
                recipients: Recipients::All,
            },
            0,
        );
        // info!("Looking for entity {}", entity_instance.identifier);
        // if let Some(path) = processor.0.get(&entity_instance.identifier) {
        //     info!("Found entity {}", entity_instance.identifier);
        //     commands.entity(entity)
        //         .insert(ScriptCollection::<LuaFile> {
        //             scripts: vec![Script::new(path.clone(),
        //                                       assets.load(path))]
        //         });
        // }
    }
}


#[derive(Clone)]
pub struct N9Level(Entity);

impl EntityRep for N9Level {
    fn entity(&self) -> Entity {
        self.0
    }
}

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
        Transform::add_fields::<'lua, Self, _>(fields);
        // fields.add_field_method_get("default", |ctx, this| Ok(N9TextStyle::default()));
    }
}
