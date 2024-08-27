use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    sprite::Anchor,
    transform::commands::PushChildInPlace,
};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods,
};

use bevy_mod_scripting::api::providers::bevy_ecs::LuaEntity;
use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::{palette::Nano9Palette, N9Image, N9Color};
use std::sync::OnceLock;

pub(crate) fn despawn_list() -> Option<&'static mut Vec<Entity>> {
    static mut MEM: OnceLock<Vec<Entity>> = OnceLock::new();
    unsafe {
        let _ = MEM.get_or_init(Vec::new);
        MEM.get_mut()
    }
}

fn despawn_list_system(mut commands: Commands) {
    if let Some(list) = despawn_list() {
        for id in list.drain(..) {
            if let Some(mut e) = commands.get_entity(id) { e.despawn() }
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, despawn_list_system);
}

#[derive(Debug, Clone, Copy)]
pub enum DropPolicy {
    Nothing,
    Despawn,
}

pub struct N9Sprite {
    pub entity: Entity,
    pub drop: DropPolicy,
}

impl Drop for N9Sprite {
    fn drop(&mut self) {
        if matches!(self.drop, DropPolicy::Despawn) {
            if let Some(list) = despawn_list() {
                list.push(self.entity);
            } else {
                warn!("Unable to despawn sprite {:?}.", self.entity);
            }
        }
    }
}

pub(crate) trait EntityRep {
    fn entity(&self) -> Entity;
}

pub(crate) trait UserDataComponent {
    fn add_fields<'lua, S: EntityRep, F: UserDataFields<'lua, S>>(fields: &mut F) {}

    fn add_methods<'lua, S: EntityRep, M: UserDataMethods<'lua, S>>(methods: &mut M) {}
}

impl<T: EntityRep> UserDataComponent for T {
    fn add_fields<'lua, S: EntityRep, F: UserDataFields<'lua, S>>(fields: &mut F) {
        fields.add_field_method_get("parent", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&Parent>> = SystemState::new(&mut world);
            let parents = system_state.get(&world);
            parents
                .get(this.entity())
                .map(|p| LuaEntity::new(p.get()))
                // .map(|p| p.get())
                .map_err(|e| LuaError::RuntimeError("No parent available".into()))
        });
        fields.add_field_method_set("parent", |ctx, this, parent: LuaEntity| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let cmd = PushChildInPlace {
                child: this.entity(),
                parent: parent.inner()?,
            };
            cmd.apply(&mut world);
            Ok(())
        });
    }
}

impl UserDataComponent for Transform {
    fn add_fields<'lua, S: EntityRep, F: UserDataFields<'lua, S>>(fields: &mut F) {
        fields.add_field_method_get("x", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&Transform>> = SystemState::new(&mut world);
            let transforms = system_state.get(&mut world);
            let transform = transforms.get(this.entity()).unwrap();
            Ok(transform.translation.x)
        });

        fields.add_field_method_set("x", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> = SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.entity()).unwrap();
            transform.translation.x = value;
            Ok(())
        });
        fields.add_field_method_get("y", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> = SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let transform = transforms.get_mut(this.entity()).unwrap();
            Ok(transform.translation.y)
        });

        fields.add_field_method_set("y", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> = SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.entity()).unwrap();
            transform.translation.y = value;
            Ok(())
        });

        fields.add_field_method_get("z", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&Transform>> = SystemState::new(&mut world);
            let transforms = system_state.get(&mut world);
            let transform = transforms.get(this.entity()).unwrap();
            Ok(transform.translation.z)
        });

        fields.add_field_method_set("z", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> = SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.entity()).unwrap();
            transform.translation.z = value;
            Ok(())
        });

        fields.add_field_method_set("parent", |ctx, this, parent: LuaEntity| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let cmd = PushChildInPlace {
                child: this.entity(),
                parent: parent.inner()?,
            };
            cmd.apply(&mut world);
            Ok(())
        });

        fields.add_field_method_get("entity", |ctx, this| Ok(LuaEntity::new(this.entity())));
    }
}

impl EntityRep for N9Sprite {
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl UserData for N9Sprite {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        Transform::add_fields::<'lua, Self, _>(fields);

        fields.add_field_method_set("color", |ctx, this, value: Option<N9Color> | {
            let world = ctx.get_world()?;
            let mut world = world.write();

            let c = value.map(|v| match v {
                N9Color::Palette(c) => Nano9Palette::get_color(c, &mut world),
                N9Color::Color(rgb) => Ok(rgb)
            }).unwrap_or(Ok(Color::WHITE))?;
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            item.color = c;
            Ok(())
        });

        fields.add_field_method_set("flip_x", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            item.flip_x = value;
            Ok(())
        });

        fields.add_field_method_set("sx", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            item.custom_size.get_or_insert(Vec2::ONE).x = value;
            Ok(())
        });

        fields.add_field_method_set("sy", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            item.custom_size.get_or_insert(Vec2::ONE).y = value;
            Ok(())
        });

        fields.add_field_method_set("flip_y", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            item.flip_y = value;
            Ok(())
        });

        fields.add_field_method_set("index", |ctx, this, value| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut TextureAtlas>> =
                SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            item.index = value;
            Ok(())
        });

        fields.add_field_method_set("anchor", |ctx, this, value: [f32; 2]| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            item.anchor = Anchor::Custom(Vec2::new(value[0] / 2.0, value[1] / 2.0));
            Ok(())
        });

        fields.add_field_method_set("vis", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Visibility>> =
                SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.entity).unwrap();
            *item = if value {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            Ok(())
        });

        fields.add_field_method_get("image", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&Handle<Image>>,)> =
                SystemState::new(&mut world);
            let (query,) = system_state.get_mut(&mut world);
            let item = query.get(this.entity).unwrap();
            Ok(N9Image {
                handle: item.clone(),
                layout: None,
            }) //.ok_or(LuaError::RuntimeError("No such image".into()))
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("despawn", |ctx, this, _: ()| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            world.despawn(this.entity);
            Ok(())
        });
        // methods.add_method_mut("set_anchor", |ctx, this, _: ()| {
        // fields.add_field_method_set("anchor", |ctx, this, value: (f32, f32)| {
        //     let world = ctx.get_world()?;
        //     let mut world = world.write();
        //     let mut system_state: SystemState<Query<&mut Sprite>> =
        //         SystemState::new(&mut world);
        //     let mut query = system_state.get_mut(&mut world);
        //     let mut item = query.get_mut(this.entity).unwrap();
        //     item.anchor = Anchor::Custom(value.0, value.1);
        //     Ok(())
        // });

        // methods.add_meta_method(MetaMethod::Add, |_, this, value: i32| {
        //     Ok(this.entity + value)
        // });
    }
}
