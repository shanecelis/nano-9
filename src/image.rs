use std::sync::{Mutex, Arc};
use bevy::{ecs::system::SystemState, prelude::*};

use crate::{palette::Nano9Palette, pixel::PixelAccess, DropPolicy, N9Color, N9Sprite, ValueExt, N9Entity};
use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{UserData, UserDataMethods};
use bevy_mod_scripting::prelude::*;

#[derive(Clone)]
pub struct N9ImageLoader;
impl FromLua<'_> for N9ImageLoader {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

// N9ImageLoad(String);

// impl Command for N9ImageLoad {
//     fn apply(self, world: &mut World) {

//     }
// }

impl UserData for N9ImageLoader {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("load", |ctx, this, path: String| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let mut system_state: SystemState<(Res<AssetServer>,)> = SystemState::new(&mut world);
            let (server,) = system_state.get(&world);
            let handle: Handle<Image> = server.load(path);
            Ok(N9Image {
                handle,
                layout: None,
            })
        });
    }
}

#[derive(Clone)]
pub struct N9Image {
    pub handle: Handle<Image>,
    pub layout: Option<Handle<TextureAtlasLayout>>,
}

impl FromLua<'_> for N9Image {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl UserData for N9Image {
    // fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
    //     fields.add_field_method_get("x", |ctx, this| {
    //         Ok(())
    //     });
    // }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut(
            "set_grid",
            |ctx, this, (width, height, columns, rows): (u32, u32, u32, u32)| {
                let world = ctx.get_world()?;
                let mut world = world.write();
                let mut system_state: SystemState<ResMut<Assets<TextureAtlasLayout>>> =
                    SystemState::new(&mut world);
                let mut layouts = system_state.get_mut(&mut world);
                this.layout = Some(layouts.add(TextureAtlasLayout::from_grid(
                    UVec2::new(width, height),
                    columns,
                    rows,
                    None,
                    None,
                )));
                Ok(())
            },
        );

        methods.add_method_mut("sprite", |ctx, this, mut args: LuaMultiValue| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            // eprintln!("args {args:?} {} ", args.len());
            let n = if args.len() == 2 {
                None
            } else {
                args.pop_front().and_then(|x| x.as_usize())
            };
            let x = args.pop_front().and_then(|v| v.to_f32()).unwrap_or(0.0);
            let y = args.pop_front().and_then(|v| v.to_f32()).unwrap_or(0.0);
            // eprintln!("x {x} y {y}");
            Ok(Arc::new(Mutex::new(if let Some(n) = n {
                N9Entity {
                    entity: world
                        .spawn((
                            SpriteBundle {
                                texture: this.handle.clone(),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            TextureAtlas {
                                layout: this.layout.clone().unwrap(),
                                index: n,
                            },
                        ))
                        .id(),
                    drop: DropPolicy::Despawn,
                }
            } else {
                N9Entity {
                    entity: world
                        .spawn((SpriteBundle {
                            texture: this.handle.clone(),
                            transform: Transform::from_xyz(x, y, 0.0),
                            ..default()
                        },))
                        .id(),
                    drop: DropPolicy::Despawn,
                }
            })))
        });

        methods.add_method_mut("spr", |ctx, this, mut args: LuaMultiValue| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            // eprintln!("args {args:?} {} ", args.len());
            let n = if args.len() == 2 {
                None
            } else {
                args.pop_front().and_then(|x| x.as_usize())
            };
            let x = args.pop_front().and_then(|v| v.to_f32()).unwrap_or(0.0);
            let y = args.pop_front().and_then(|v| v.to_f32()).unwrap_or(0.0);
            // eprintln!("x {x} y {y}");
            Ok(Arc::new(Mutex::new(if let Some(n) = n {
                N9Sprite {
                    entity: world
                        .spawn((
                            SpriteBundle {
                                texture: this.handle.clone(),
                                transform: Transform::from_xyz(x, y, 0.0),
                                ..default()
                            },
                            TextureAtlas {
                                layout: this.layout.clone().unwrap(),
                                index: n,
                            },
                        ))
                        .id(),
                    drop: DropPolicy::Despawn,
                }
            } else {
                N9Sprite {
                    entity: world
                        .spawn((SpriteBundle {
                            texture: this.handle.clone(),
                            transform: Transform::from_xyz(x, y, 0.0),
                            ..default()
                        },))
                        .id(),
                    drop: DropPolicy::Despawn,
                }
            })))
        });

        methods.add_method_mut("set_pixel", |ctx, this, (x, y, c): (f32, f32, N9Color)| {
            let world = ctx.get_world()?;
            let mut world = world.write();
            let color = match c {
                N9Color::Palette(c) => Nano9Palette::get_color(c, &mut world)?,
                N9Color::Color(rgb) => rgb.into(),
            };
            let mut system_state: SystemState<(ResMut<Assets<Image>>,)> =
                SystemState::new(&mut world);
            let (mut images,) = system_state.get_mut(&mut world);
            let image = images.get_mut(&this.handle).unwrap();
            let height = image.texture_descriptor.size.height;
            let _ = image.set_pixel((x as usize, (height as f32 - y) as usize), color);
            Ok(())
        });

        // methods.add_method("get_pixel", |ctx, this, (x, y, c): (f32, f32, Value)| {
        //     let world = ctx.get_world()?;
        //     let mut world = world.write();
        //     let color = Nano9Palette::get_color(c, &mut world);
        //     let mut system_state: SystemState<(
        //         ResMut<Assets<Image>>,
        //     )> = SystemState::new(&mut world);
        //     let (images) = system_state.get(&mut world);
        //     let image = images.get(&this.handle).unwrap();
        //     let color = image.get_pixel((x as usize, y as usize));
        //     Ok(())
        // });
    }
}

#[cfg(test)]
mod test {
    use bevy_mod_scripting::prelude::*;
    #[test]
    fn explore_multivalue() {
        let mut m = LuaMultiValue::new();
        m.push_front(LuaValue::Boolean(true));
        // m.push_back(true);
        assert_eq!(m.pop_front().unwrap(), LuaValue::Boolean(true));
    }
}
