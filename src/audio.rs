use std::sync::Arc;
use bevy::{audio::PlaybackMode, ecs::system::SystemState, prelude::*};

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;
// use bevy_pixel_buffer::prelude::*;
use crate::despawn_list;

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
            let mut system_state: SystemState<(Res<AssetServer>,)> = SystemState::new(&mut world);
            let (server,) = system_state.get(&world);
            let handle: Handle<AudioSource> = server.load(path);
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
            let id = world
                .spawn(AudioBundle {
                    source: this.handle.clone(),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        ..default()
                    },
                })
                .id();
            Ok(Arc::new(N9Sound(id)))
        });
        methods.add_method_mut("play_loop", |ctx, this, _: ()| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let id = world
                .spawn(AudioBundle {
                    source: this.handle.clone(),
                    settings: PlaybackSettings {
                        mode: PlaybackMode::Loop,
                        ..default()
                    },
                })
                .id();
            Ok(Arc::new(N9Sound(id)))
        });
    }
}

pub struct N9Sound(pub Entity);

impl Drop for N9Sound {
    fn drop(&mut self) {
        if let Some(list) = despawn_list() {
            list.push(self.0);
        } else {
            warn!("Unable to despawn sprite {:?}.", self.0);
        }
    }
}

impl UserData for N9Sound {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("vol", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);
            let query = system_state.get(&mut world);
            let sink = query.get(this.0).unwrap();
            Ok(sink.volume())
        });

        fields.add_field_method_set("vol", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let sink = query.get_mut(this.0).unwrap();
            sink.set_volume(value);
            Ok(())
        });

        fields.add_field_method_get("speed", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);
            let query = system_state.get(&world);
            let sink = query.get(this.0).unwrap();
            Ok(sink.speed())
        });

        fields.add_field_method_set("speed", |ctx, this, value: f32| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let sink = query.get_mut(this.0).unwrap();
            sink.set_speed(value);
            Ok(())
        });

        fields.add_field_method_get("is_playing", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);
            let query = system_state.get(&world);
            Ok(query
                .get(this.0)
                .map(|sink| !sink.is_paused() && !sink.empty())
                .unwrap_or(false))
        });

        fields.add_field_method_get("pause", |ctx, this| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);
            let query = system_state.get(&world);
            let sink = query.get(this.0).unwrap();
            Ok(sink.is_paused())
        });

        fields.add_field_method_set("pause", |ctx, this, value: bool| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);

            let mut query = system_state.get_mut(&mut world);
            let sink = query.get_mut(this.0).unwrap();
            if value {
                sink.pause();
            } else {
                sink.play();
            }
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

        methods.add_method_mut("stop", |ctx, this, _: ()| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<Query<&AudioSink>> = SystemState::new(&mut world);
            let mut query = system_state.get_mut(&mut world);
            let sink = query.get_mut(this.0).unwrap();
            sink.stop();
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
