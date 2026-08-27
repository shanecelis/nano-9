//! [`ScriptValue`] ↔ luars conversions.

use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

use bevy_mod_scripting_asset::Language;
use bevy_mod_scripting_bindings::{
    InteropError, VariadicTuple, function::script_function::FunctionCallContext,
    script_value::ScriptValue,
};
use bevy_platform::collections::HashMap;
use luars::{
    FromLua, FromLuaMulti, IntoLua, LuaApi, LuaError, LuaFunction, LuaResult, LuaState, LuaValue,
};

use crate::reference::{BoundScriptFunction, LuaReflectReference, LuaStaticReflectReference};

/// Caller context used for Lua → BMS function dispatch.
pub const LUA_CALLER_CONTEXT: FunctionCallContext = FunctionCallContext::new(Language::Lua);

/// Wrapper around many [`ScriptValue`]s for Lua multi-return.
pub struct MultiLuaScriptValue(pub VecDeque<ScriptValue>);

impl MultiLuaScriptValue {
    pub fn into_script_value(mut self) -> ScriptValue {
        if self.0.is_empty() {
            ScriptValue::Unit
        } else if self.0.len() == 1 {
            self.0.pop_front().unwrap_or(ScriptValue::Unit)
        } else {
            ScriptValue::Tuple(VariadicTuple(self.0))
        }
    }

    pub fn from_script_value(value: ScriptValue) -> Self {
        if let ScriptValue::Tuple(VariadicTuple(tuple)) = value {
            Self(tuple)
        } else {
            MultiLuaScriptValue(VecDeque::from_iter([value]))
        }
    }
}

impl FromLuaMulti for MultiLuaScriptValue {
    fn from_lua_multi(values: Vec<LuaValue>, state: &mut LuaState) -> Result<Self, String> {
        let mut vals = VecDeque::with_capacity(values.len());
        for val in values {
            vals.push_back(LuaScriptValue::from_lua(val, state)?.0);
        }
        Ok(MultiLuaScriptValue(vals))
    }
}

/// A [`ScriptValue`] that converts through luars [`FromLua`] / [`IntoLua`].
#[derive(Debug, Clone)]
pub struct LuaScriptValue(pub ScriptValue);

impl Deref for LuaScriptValue {
    type Target = ScriptValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LuaScriptValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<ScriptValue> for LuaScriptValue {
    fn from(value: ScriptValue) -> Self {
        LuaScriptValue(value)
    }
}

impl From<LuaScriptValue> for ScriptValue {
    fn from(value: LuaScriptValue) -> Self {
        value.0
    }
}

impl FromLua for LuaScriptValue {
    fn from_lua(value: LuaValue, state: &mut LuaState) -> Result<Self, String> {
        lua_value_to_script(state, value).map(LuaScriptValue)
    }
}

impl IntoLua for LuaScriptValue {
    fn into_lua(self, state: &mut LuaState) -> Result<usize, String> {
        script_value_into_lua(state, self.0).map_err(|e| match e {
            LuaError::RuntimeError => "lua runtime error".to_owned(),
            other => format!("{other}"),
        })
    }
}

pub fn lua_value_to_script(state: &mut LuaState, value: LuaValue) -> Result<ScriptValue, String> {
    if value.is_nil() {
        return Ok(ScriptValue::Unit);
    }
    if let Some(b) = value.as_boolean() {
        return Ok(ScriptValue::Bool(b));
    }
    if let Some(i) = value.as_integer_strict() {
        return Ok(ScriptValue::Integer(i));
    }
    if let Some(n) = value.as_number() {
        return Ok(ScriptValue::Float(n));
    }
    if let Some(s) = value.as_str() {
        return Ok(ScriptValue::String(s.to_owned().into()));
    }
    if value.is_function() || value.is_c_callable() {
        let func = LuaFunction::from_lua(value, state)?;
        return Ok(ScriptValue::Function(
            (move |_context: FunctionCallContext, args: VecDeque<ScriptValue>| {
                let mapped: Vec<LuaScriptValue> = args.into_iter().map(LuaScriptValue).collect();
                match func.call::<_, MultiLuaScriptValue>(mapped) {
                    Ok(v) => v.into_script_value(),
                    Err(e) => ScriptValue::Error(InteropError::external(e)),
                }
            })
            .into(),
        ));
    }
    if let Some(table) = value.as_table() {
        let entries = table.iter_all();
        if entries.is_empty() {
            return Ok(ScriptValue::List(VecDeque::new()));
        }
        let all_string_keys = entries.iter().all(|(k, _)| k.as_str().is_some());
        if all_string_keys {
            let mut map = HashMap::new();
            for (k, v) in entries {
                let key = k
                    .as_str()
                    .ok_or_else(|| "expected string table key".to_owned())?
                    .to_owned();
                map.insert(key, lua_value_to_script(state, v)?);
            }
            return Ok(ScriptValue::Map(map));
        }
        let mut items: Vec<(i64, ScriptValue)> = Vec::new();
        for (k, v) in entries {
            let idx = k
                .as_integer()
                .ok_or_else(|| format!("unsupported table key type {}", k.type_name()))?;
            items.push((idx, lua_value_to_script(state, v)?));
        }
        items.sort_by_key(|(i, _)| *i);
        return Ok(ScriptValue::List(
            items.into_iter().map(|(_, v)| v).collect(),
        ));
    }
    if value.is_userdata() {
        if let Some(ud) = value.as_userdata_mut() {
            if let Some(r) = ud.downcast_ref::<LuaReflectReference>() {
                return Ok(ScriptValue::Reference(r.0.clone()));
            }
            if let Some(bound) = ud.downcast_ref::<BoundScriptFunction>() {
                return Ok(bound.clone().into_script_value());
            }
            let _ = ud.downcast_ref::<LuaStaticReflectReference>();
        }
        return Err("unsupported userdata type".to_owned());
    }
    Err(format!("unsupported lua value type {}", value.type_name()))
}

pub fn script_value_into_lua(state: &mut LuaState, value: ScriptValue) -> LuaResult<usize> {
    match value {
        ScriptValue::Unit => {
            state.push_value(LuaValue::nil())?;
            Ok(1)
        }
        ScriptValue::Bool(b) => {
            state.push_value(LuaValue::boolean(b))?;
            Ok(1)
        }
        ScriptValue::Integer(i) => {
            state.push_value(LuaValue::integer(i))?;
            Ok(1)
        }
        ScriptValue::Float(f) => {
            state.push_value(LuaValue::float(f))?;
            Ok(1)
        }
        ScriptValue::String(s) => s.into_owned().into_lua(state).map_err(|m| state.error(m)),
        ScriptValue::Error(e) => Err(state.error(e.to_string())),
        ScriptValue::Reference(r) => LuaReflectReference(r)
            .into_lua(state)
            .map_err(|m| state.error(m)),
        ScriptValue::Function(f) => BoundScriptFunction::Fn(f)
            .into_lua(state)
            .map_err(|m| state.error(m)),
        ScriptValue::FunctionMut(f) => BoundScriptFunction::FnMut(f)
            .into_lua(state)
            .map_err(|m| state.error(m)),
        ScriptValue::List(list) | ScriptValue::Tuple(VariadicTuple(list)) => {
            let table = <LuaState as LuaApi>::create_sequence_from(
                state,
                list.into_iter().map(LuaScriptValue),
            )?;
            table.into_lua(state).map_err(|m| state.error(m))
        }
        ScriptValue::Map(map) => {
            let table = <LuaState as LuaApi>::create_table_from(
                state,
                map.into_iter().map(|(k, v)| (k, LuaScriptValue(v))),
            )?;
            table.into_lua(state).map_err(|m| state.error(m))
        }
    }
}

impl IntoLua for LuaReflectReference {
    fn into_lua(self, state: &mut LuaState) -> Result<usize, String> {
        let ud =
            <LuaState as LuaApi>::create_userdata(state, self).map_err(|e| format!("{e:?}"))?;
        ud.into_lua(state)
    }
}

impl IntoLua for LuaStaticReflectReference {
    fn into_lua(self, state: &mut LuaState) -> Result<usize, String> {
        let ud =
            <LuaState as LuaApi>::create_userdata(state, self).map_err(|e| format!("{e:?}"))?;
        ud.into_lua(state)
    }
}

impl IntoLua for BoundScriptFunction {
    fn into_lua(self, state: &mut LuaState) -> Result<usize, String> {
        let ud =
            <LuaState as LuaApi>::create_userdata(state, self).map_err(|e| format!("{e:?}"))?;
        ud.into_lua(state)
    }
}
