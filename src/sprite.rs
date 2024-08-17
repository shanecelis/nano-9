
use bevy::{
    ecs::system::SystemState,
    sprite::Anchor,
    prelude::*,
};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::{
    palette::Nano9Palette,
};
use std::sync::OnceLock;

pub(crate) fn despawn_list() -> Option<&'static mut Vec<Entity>> {
    static mut MEM: OnceLock<Vec<Entity>> = OnceLock::new();
    unsafe {
        let _ = MEM.get_or_init(|| Vec::new());
        MEM.get_mut()
    }
}

fn despawn_list_system(mut commands: Commands) {
    if let Some(list) = despawn_list() {
        for id in list.drain(..) {
            commands.get_entity(id).map(|mut e| e.despawn());
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, despawn_list_system);
}

pub struct MySprite(pub Entity);

impl Drop for MySprite {
    fn drop(&mut self) {
        if let Some(list) = despawn_list() {
            list.push(self.0);
        } else {
            warn!("Unable to despawn sprite {:?}.", self.0);
        }
    }
}


impl UserData for MySprite {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&Transform>> = SystemState::new(&mut world);
            let transforms = system_state.get(&mut world);
            let transform = transforms.get(this.0).unwrap();
            Ok(transform.translation.x)
        });

        fields.add_field_method_set("x", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> =
                SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.0).unwrap();
            transform.translation.x = value;
            Ok(())
        });


        fields.add_field_method_get("y", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> =
                SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let transform = transforms.get_mut(this.0).unwrap();
            Ok(transform.translation.y)
        });

        fields.add_field_method_set("y", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> =
                SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.0).unwrap();
            transform.translation.y = value;
            Ok(())
        });

        fields.add_field_method_get("z", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&Transform>> = SystemState::new(&mut world);
            let transforms = system_state.get(&mut world);
            let transform = transforms.get(this.0).unwrap();
            Ok(transform.translation.z)
        });

        fields.add_field_method_set("z", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Transform>> =
                SystemState::new(&mut world);
            let mut transforms = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.0).unwrap();
            transform.translation.z = value;
            Ok(())
        });
        fields.add_field_method_set("color", |ctx, this, value| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let c = if value == Value::Nil {
                Color::WHITE
            } else {
                Nano9Palette::get_color(value, &mut world)
            };
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.color = c;
            Ok(())
        });

        fields.add_field_method_set("flip_x", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.flip_x = value;
            Ok(())
        });

        fields.add_field_method_set("sx", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.custom_size.get_or_insert(Vec2::ONE).x = value;
            Ok(())
        });

        fields.add_field_method_set("sy", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.custom_size.get_or_insert(Vec2::ONE).y = value;
            Ok(())
        });

        fields.add_field_method_set("flip_y", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.flip_y = value;
            Ok(())
        });

        fields.add_field_method_set("index", |ctx, this, value| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut TextureAtlas>> =
                SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.index = value;
            Ok(())
        });

        fields.add_field_method_set("anchor", |ctx, this, value: [f32; 2]| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Sprite>> =
                SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.anchor = Anchor::Custom(Vec2::new(value[0] / 2.0, value[1] / 2.0));
            Ok(())
        });

        fields.add_field_method_set("vis", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&mut Visibility>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            *item = if value { Visibility::Visible } else { Visibility::Hidden };
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("despawn", |ctx, this, _: ()| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            world.despawn(this.0);
            Ok(())
        });
        // methods.add_method_mut("set_anchor", |ctx, this, _: ()| {
        // fields.add_field_method_set("anchor", |ctx, this, value: (f32, f32)| {
        //     let world = ctx.get_world()?;
        //     let mut world = world.write();
        //     let mut system_state: SystemState<Query<&mut Sprite>> =
        //         SystemState::new(&mut world);
        //     let mut query = system_state.get_mut(&mut world);
        //     let mut item = query.get_mut(this.0).unwrap();
        //     item.anchor = Anchor::Custom(value.0, value.1);
        //     Ok(())
        // });

        // methods.add_meta_method(MetaMethod::Add, |_, this, value: i32| {
        //     Ok(this.0 + value)
        // });
    }
}
