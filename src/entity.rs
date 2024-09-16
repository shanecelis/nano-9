use bevy::{
    ecs::{system::SystemState, world::Command},
    prelude::*,
    sprite::Anchor,
    transform::commands::PushChildInPlace,
};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods, MetaMethod
};

use bevy_mod_scripting::api::{
    providers::bevy_ecs::LuaEntity,
    common::bevy::ScriptWorld,
};
use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::{palette::Nano9Palette, N9Color, N9Image, DropPolicy};
use std::sync::OnceLock;

pub struct N9Entity {
    pub entity: Entity,
    pub drop: DropPolicy,
}

impl UserData for N9Entity {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        // Transform::add_fields::<'lua, Self, _>(fields);
        //
        //
        fields.add_field_method_get("name", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&Name>> = SystemState::new(&mut world);
            let items = system_state.get(&mut world);
            Ok(items.get(this.entity).map(|name| name.as_str().to_owned()).ok())
        });
        fields.add_field_method_set("name", |ctx, this, value: String| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut commands = world.commands();
            commands.entity(this.entity).insert(Name::new(value));
            Ok(())
        });

        fields.add_field_method_get("sprite", |ctx, this| {
            let world = ctx.get_world()?;
            let world = ScriptWorld::new(world);
            // let mut world = world.write();
            let t = world.get_type_by_name("Sprite").unwrap();
            world.get_component(this.entity, t)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        fields.add_field_method_get("transform", |ctx, this| {
            let world = ctx.get_world()?;
            let world = ScriptWorld::new(world);
            // let mut world = world.write();
            let t = world.get_type_by_name("Transform").unwrap();
            world.get_component(this.entity, t)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Index, |ctx, this, index: String| {

            let world = ctx.get_world()?;
            let world = ScriptWorld::new(world);
            // let mut world = world.write();
            let t = world.get_type_by_name(&index).unwrap();
            world.get_component(this.entity, t)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    }
}
