//! Reflect userdata wrappers for luars.

use std::any::{Any, TypeId};
use std::collections::VecDeque;

use bevy_mod_scripting_bindings::{
    InteropError, ReflectReference, WorldExtensions,
    function::script_function::{DynamicScriptFunction, DynamicScriptFunctionMut},
    script_value::ScriptValue,
};
use bevy_mod_scripting_bindings_domain::ScriptOperatorNames;
use bevy_mod_scripting_display::OrFakeId;
use bevy_mod_scripting_world::ThreadWorldContainer;
use luars::{CFunction, LuaResult, LuaState, UdValue, UserDataTrait};

use crate::script_value::{LUA_CALLER_CONTEXT, lua_value_to_script, script_value_into_lua};

/// Userdata wrapper around [`ReflectReference`].
#[derive(Debug, Clone)]
pub struct LuaReflectReference(pub ReflectReference);

/// Static type handle so scripts can call `Entity.from_raw(...)`.
#[derive(Debug, Clone, Copy)]
pub struct LuaStaticReflectReference(pub TypeId);

/// Callable userdata wrapping a BMS dynamic function.
#[derive(Clone)]
pub enum BoundScriptFunction {
    Fn(DynamicScriptFunction),
    FnMut(DynamicScriptFunctionMut),
}

impl BoundScriptFunction {
    pub fn into_script_value(self) -> ScriptValue {
        match self {
            BoundScriptFunction::Fn(f) => ScriptValue::Function(f),
            BoundScriptFunction::FnMut(f) => ScriptValue::FunctionMut(f),
        }
    }

    fn call_with(&self, state: &mut LuaState, args: VecDeque<ScriptValue>) -> LuaResult<usize> {
        let out = match self {
            BoundScriptFunction::Fn(f) => f.call(args, LUA_CALLER_CONTEXT),
            BoundScriptFunction::FnMut(f) => f.call(args, LUA_CALLER_CONTEXT),
        };
        match out {
            Ok(v) => script_value_into_lua(state, v),
            Err(e) => Err(state.error(e.to_string())),
        }
    }
}

fn bound_script_call(state: &mut LuaState) -> LuaResult<usize> {
    let self_val = state
        .get_arg(1)
        .ok_or_else(|| state.error("missing function self".to_owned()))?;
    let bound = self_val
        .as_userdata_mut()
        .and_then(|ud| ud.downcast_ref::<BoundScriptFunction>())
        .cloned()
        .ok_or_else(|| state.error("expected BoundScriptFunction".to_owned()))?;

    let n = state.get_args().len();
    let mut args = VecDeque::new();
    for i in 2..=n {
        if let Some(v) = state.get_arg(i) {
            let sv = lua_value_to_script(state, v).map_err(|m| state.error(m))?;
            args.push_back(sv);
        }
    }
    bound.call_with(state, args)
}

impl UserDataTrait for BoundScriptFunction {
    fn type_name(&self) -> &'static str {
        "BmsFunction"
    }

    fn lua_call(&self) -> Option<CFunction> {
        Some(bound_script_call)
    }

    fn lua_tostring(&self) -> Option<String> {
        Some("function".to_owned())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn script_to_ud(value: ScriptValue) -> Option<UdValue> {
    match value {
        ScriptValue::Unit => Some(UdValue::Nil),
        ScriptValue::Bool(b) => Some(UdValue::Boolean(b)),
        ScriptValue::Integer(i) => Some(UdValue::Integer(i)),
        ScriptValue::Float(f) => Some(UdValue::Number(f)),
        ScriptValue::String(s) => Some(UdValue::Str(s.into_owned())),
        ScriptValue::Reference(r) => Some(UdValue::from_userdata(LuaReflectReference(r))),
        ScriptValue::Function(f) => Some(UdValue::from_userdata(BoundScriptFunction::Fn(f))),
        ScriptValue::FunctionMut(f) => Some(UdValue::from_userdata(BoundScriptFunction::FnMut(f))),
        ScriptValue::Error(_)
        | ScriptValue::List(_)
        | ScriptValue::Tuple(_)
        | ScriptValue::Map(_) => None,
    }
}

fn ud_operand_to_script(other: &UdValue) -> Option<ScriptValue> {
    if let Some(r) = other.as_userdata_ref::<LuaReflectReference>() {
        return Some(ScriptValue::Reference(r.0.clone()));
    }
    match other {
        UdValue::Nil => Some(ScriptValue::Unit),
        UdValue::Boolean(b) => Some(ScriptValue::Bool(*b)),
        UdValue::Integer(i) => Some(ScriptValue::Integer(*i)),
        UdValue::Number(n) => Some(ScriptValue::Float(*n)),
        UdValue::Str(s) => Some(ScriptValue::String(s.clone().into())),
        _ => None,
    }
}

fn reflect_binop(
    this: &ReflectReference,
    other: &UdValue,
    op: ScriptOperatorNames,
) -> Option<UdValue> {
    let world = ThreadWorldContainer.try_get_context().ok()?.world;
    let other = ud_operand_to_script(other)?;
    let target_type_id = this.tail_type_id(world.clone()).ok()?.or_fake_id();
    let args = vec![ScriptValue::Reference(this.clone()), other];
    let out = world
        .try_call_overloads(
            target_type_id,
            op.script_function_name(),
            args,
            LUA_CALLER_CONTEXT,
        )
        .ok()?;
    script_to_ud(out)
}

impl UserDataTrait for LuaReflectReference {
    fn type_name(&self) -> &'static str {
        "ReflectReference"
    }

    fn get_field(&self, key: &str) -> Option<UdValue> {
        let world = ThreadWorldContainer.try_get_context().ok()?.world;
        let type_id = self.0.tail_type_id(world.clone()).ok()?.or_fake_id();
        match world.lookup_function([type_id, TypeId::of::<ReflectReference>()], key.to_owned()) {
            Ok(func) => Some(UdValue::from_userdata(BoundScriptFunction::Fn(func))),
            Err(name) => {
                let registry = world.script_function_registry();
                let registry = registry.read();
                let out = registry
                    .magic_functions
                    .get(
                        LUA_CALLER_CONTEXT,
                        self.0.clone(),
                        ScriptValue::String(name),
                    )
                    .ok()?;
                script_to_ud(out)
            }
        }
    }

    fn set_field(&mut self, key: &str, value: UdValue) -> Option<Result<(), String>> {
        let world = match ThreadWorldContainer.try_get_context() {
            Ok(c) => c.world,
            Err(e) => return Some(Err(format!("{e:?}"))),
        };
        let value = match ud_operand_to_script(&value) {
            Some(v) => v,
            None => return Some(Err("unsupported assignment value".to_owned())),
        };
        let registry = world.script_function_registry();
        let registry = registry.read();
        match registry.magic_functions.set(
            LUA_CALLER_CONTEXT,
            self.0.clone(),
            ScriptValue::String(key.to_owned().into()),
            value,
        ) {
            Ok(()) => Some(Ok(())),
            Err(e) => Some(Err(e.to_string())),
        }
    }

    fn lua_tostring(&self) -> Option<String> {
        let world = ThreadWorldContainer.try_get_context().ok()?.world;
        let func = world
            .lookup_function(
                [TypeId::of::<ReflectReference>()],
                ScriptOperatorNames::DisplayPrint.script_function_name(),
            )
            .ok()?;
        match func.call(
            vec![ScriptValue::Reference(self.0.clone())],
            LUA_CALLER_CONTEXT,
        ) {
            Ok(ScriptValue::String(s)) => Some(s.into_owned()),
            Ok(other) => Some(format!("{other:?}")),
            Err(_) => None,
        }
    }

    fn lua_add(&self, other: &UdValue) -> Option<UdValue> {
        reflect_binop(&self.0, other, ScriptOperatorNames::Addition)
    }
    fn lua_sub(&self, other: &UdValue) -> Option<UdValue> {
        reflect_binop(&self.0, other, ScriptOperatorNames::Subtraction)
    }
    fn lua_mul(&self, other: &UdValue) -> Option<UdValue> {
        reflect_binop(&self.0, other, ScriptOperatorNames::Multiplication)
    }
    fn lua_div(&self, other: &UdValue) -> Option<UdValue> {
        reflect_binop(&self.0, other, ScriptOperatorNames::Division)
    }
    fn lua_mod(&self, other: &UdValue) -> Option<UdValue> {
        reflect_binop(&self.0, other, ScriptOperatorNames::Remainder)
    }
    fn lua_pow(&self, other: &UdValue) -> Option<UdValue> {
        reflect_binop(&self.0, other, ScriptOperatorNames::Exponentiation)
    }
    fn lua_unm(&self) -> Option<UdValue> {
        let world = ThreadWorldContainer.try_get_context().ok()?.world;
        let target_type_id = self.0.tail_type_id(world.clone()).ok()?.or_fake_id();
        let out = world
            .try_call_overloads(
                target_type_id,
                ScriptOperatorNames::Negation.script_function_name(),
                vec![ScriptValue::Reference(self.0.clone())],
                LUA_CALLER_CONTEXT,
            )
            .ok()?;
        script_to_ud(out)
    }
    fn lua_eq(&self, other: &dyn UserDataTrait) -> Option<bool> {
        let other = other.as_any().downcast_ref::<LuaReflectReference>()?;
        Some(self.0 == other.0)
    }
    fn lua_len(&self) -> Option<UdValue> {
        let world = ThreadWorldContainer.try_get_context().ok()?.world;
        self.0
            .len(world)
            .ok()
            .flatten()
            .map(|n| UdValue::Integer(n as i64))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl UserDataTrait for LuaStaticReflectReference {
    fn type_name(&self) -> &'static str {
        "StaticReflectReference"
    }

    fn get_field(&self, key: &str) -> Option<UdValue> {
        let world = ThreadWorldContainer.try_get_context().ok()?.world;
        match world.lookup_function([self.0], key.to_owned()) {
            Ok(func) => Some(UdValue::from_userdata(BoundScriptFunction::Fn(func))),
            Err(_) => None,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Convert an interop error into a Lua runtime error.
pub fn interop_to_lua(e: InteropError) -> String {
    e.to_string()
}
