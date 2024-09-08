use bevy::prelude::*;
use std::sync::Arc;

use crate::ValueExt;
use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{
    UserData, UserDataFields, UserDataMethods,
};
use bevy_mod_scripting::prelude::*;

#[derive(Debug, Clone)]
pub enum N9Color {
    Palette(usize),
    Color(LinearRgba),
}

impl FromLua<'_> for N9Color {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        fn bad_arg(s: &str) -> LuaError {
            LuaError::WithContext {
                context: format!("unable to convert {s:?} field to f32."),
                cause: Arc::new(LuaError::UserDataTypeMismatch),
            }
        }
        match value {
            Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
            Value::Integer(n) => Ok(N9Color::Palette(n as usize)),
            Value::Number(n) => Ok(N9Color::Palette(n as usize)),
            Value::Table(t) => {
                let l = t.len().unwrap_or(0);
                if t.contains_key("r")? && t.contains_key("g")? && t.contains_key("b")? {
                    Ok(N9Color::Color(LinearRgba::new(
                        t.get("r")
                            .and_then(|x: Value| x.to_f32().ok_or(bad_arg("r")))?,
                        t.get("g")
                            .and_then(|x: Value| x.to_f32().ok_or(bad_arg("g")))?,
                        t.get("b")
                            .and_then(|x: Value| x.to_f32().ok_or(bad_arg("b")))?,
                        t.get("a").map(|x: Value| x.as_f32().unwrap_or(1.0))?,
                    )))
                } else if l >= 3 {
                    Ok(N9Color::Color(LinearRgba::new(
                        t.get(1)
                            .and_then(|x: Value| x.to_f32().ok_or(bad_arg("r")))?,
                        t.get(2)
                            .and_then(|x: Value| x.to_f32().ok_or(bad_arg("g")))?,
                        t.get(3)
                            .and_then(|x: Value| x.to_f32().ok_or(bad_arg("b")))?,
                        t.get(4).map(|x: Value| x.as_f32().unwrap_or(1.0))?,
                    )))
                } else {
                    Err(LuaError::UserDataTypeMismatch)
                }
            }
            _ => unreachable!(),
        }
    }
}

impl UserData for N9Color {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("i", |ctx, this| match this {
            Self::Palette(i) => Ok(Value::Integer(*i as i64)),
            Self::Color(_) => Ok(Value::Nil),
        });
        fields.add_field_method_set("i", |ctx, this, value: usize| match this {
            Self::Palette(ref mut i) => {
                *i = value;
                Ok(())
            }
            Self::Color(_) => Err(LuaError::SyntaxError {
                message: "Cannot set index of RGBA color".into(),
                incomplete_input: false,
            }),
        });
        fields.add_field_method_get("r", |ctx, this| match this {
            Self::Palette(_) => Ok(Value::Nil),
            Self::Color(c) => Ok(Value::Number(c.red as f64)),
        });

        fields.add_field_method_set("r", |ctx, this, value: f32| match this {
            Self::Palette(_) => Err(LuaError::RuntimeError(
                "Cannot set red channel of palette color".into(),
            )),
            Self::Color(c) => {
                c.red = value;
                Ok(())
            }
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // methods.add_method_mut(
        //     "set_grid",
        //     |ctx, this, (width, height, columns, rows): (f32, f32, usize, usize)| {
        //         let world = ctx.get_world()?;
        //         let mut world = world.write();
        //         let mut system_state: SystemState<ResMut<Assets<TextureAtlasLayout>>> =
        //             SystemState::new(&mut world);
        //         let mut layouts = system_state.get_mut(&mut world);
        //         this.layout = Some(layouts.add(TextureAtlasLayout::from_grid(
        //             Vec2::new(width, height),
        //             columns,
        //             rows,
        //             None,
        //             None,
        //         )));
        //         Ok(())
        //     },
        // );
    }
}
