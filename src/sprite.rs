use std::sync::Mutex;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::system::SystemState,
    prelude::*,
    reflect::Reflect,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    utils::Duration,
    window::PresentMode,
    window::{PrimaryWindow, WindowResized, WindowResolution},
};

use bevy_asset_loader::prelude::*;
use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    MetaMethod, UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::{
    assets::{self, ImageHandles},
    pixel::PixelAccess,
    screens,
    palette::Nano9Palette,
};


pub struct MySprite(pub Entity);

// impl Drop for MySprite {
//     fn drop(&mut self) {
//         eprintln!("Blah");
//     }
// }

impl UserData for MySprite {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&Transform>)> = SystemState::new(&mut world);
            let (transforms) = system_state.get(&mut world);
            let transform = transforms.get(this.0).unwrap();
            Ok(transform.translation.x)
        });

        fields.add_field_method_set("x", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut Transform>)> =
                SystemState::new(&mut world);
            let (mut transforms) = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.0).unwrap();
            transform.translation.x = value;
            Ok(())
        });

        fields.add_field_method_get("y", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut Transform>)> =
                SystemState::new(&mut world);
            let (mut transforms) = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.0).unwrap();
            Ok(-transform.translation.y)
        });

        fields.add_field_method_set("y", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut Transform>)> =
                SystemState::new(&mut world);
            let (mut transforms) = system_state.get_mut(&mut world);
            let mut transform = transforms.get_mut(this.0).unwrap();
            transform.translation.y = -value;
            Ok(())
        });

        fields.add_field_method_set("color", |ctx, this, value| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let c = Nano9Palette::get_color(value, &mut world);
            let mut system_state: SystemState<(Query<&mut Sprite>)> = SystemState::new(&mut world);
            let (mut query) = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.color = c;
            Ok(())
        });

        fields.add_field_method_set("flip_x", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut Sprite>)> = SystemState::new(&mut world);
            let (mut query) = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.flip_x = value;
            Ok(())
        });

        fields.add_field_method_set("sx", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut Sprite>)> = SystemState::new(&mut world);
            let (mut query) = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.custom_size.get_or_insert(Vec2::ONE).x = value;
            Ok(())
        });

        fields.add_field_method_set("sy", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut Sprite>)> = SystemState::new(&mut world);
            let (mut query) = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.custom_size.get_or_insert(Vec2::ONE).y = value;
            Ok(())
        });

        fields.add_field_method_set("flip_y", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut Sprite>)> = SystemState::new(&mut world);
            let (mut query) = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.flip_y = value;
            Ok(())
        });

        fields.add_field_method_set("index", |ctx, this, value| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Query<&mut TextureAtlas>)> =
                SystemState::new(&mut world);
            let (mut query) = system_state.get_mut(&mut world);
            let mut item = query.get_mut(this.0).unwrap();
            item.index = value;
            Ok(())
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("drop", |ctx, this, value: ()| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            world.despawn(this.0);
            Ok(())
        });

        // methods.add_meta_method(MetaMethod::Add, |_, this, value: i32| {
        //     Ok(this.0 + value)
        // });
    }
}
