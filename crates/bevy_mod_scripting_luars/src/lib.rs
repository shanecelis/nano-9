//! luars (Lua 5.5) backend for bevy_mod_scripting.
//!
//! Lives outside the BMS repo as an `IntoScriptPluginParams` plugin crate.

use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
    str,
    sync::Arc,
};

use bevy_app::{App, Plugin};
use bevy_asset::Handle;
use bevy_ecs::{
    entity::Entity,
    world::{Mut, World, WorldId},
};
use bevy_log::trace;
use bevy_mod_scripting_asset::{Language, ScriptAsset};
use bevy_mod_scripting_bindings::{
    InteropError, PartialReflectExt, WorldExtensions, function::namespace::Namespace,
    globals::AppScriptGlobalsRegistry, script_value::ScriptValue,
};
use bevy_mod_scripting_core::{
    IntoScriptPluginParams, ScriptingPlugin,
    callbacks::ScriptCallbacks,
    config::{GetPluginThreadConfig, ScriptingPluginConfiguration},
    event::CallbackLabel,
    make_plugin_config_static,
    script::ContextPolicy,
};
use bevy_mod_scripting_script::ScriptAttachment;
use bevy_mod_scripting_world::ThreadWorldContainer;
use luars::{Lua, LuaApi, LuaError, LuaFunction, LuaResult, SafeOption, Stdlib};

pub mod reference;
pub mod script_value;

pub use luars;
pub use reference::{LuaReflectReference, LuaStaticReflectReference};
pub use script_value::{LUA_CALLER_CONTEXT, LuaScriptValue, MultiLuaScriptValue};

make_plugin_config_static!(LuarsScriptingPlugin);

/// Lua VM handle stored as the BMS context.
pub struct LuarsContext {
    pub lua: Lua,
    pub last_loaded_script_name: Option<String>,
}

impl Deref for LuarsContext {
    type Target = Lua;
    fn deref(&self) -> &Self::Target {
        &self.lua
    }
}

impl DerefMut for LuarsContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lua
    }
}

impl LuarsContext {
    fn new() -> Result<Self, InteropError> {
        let mut lua = Lua::new(SafeOption::default());
        open_host_stdlibs(&mut lua).map_err(|e| lua_to_interop(&mut lua, e))?;
        Ok(Self {
            lua,
            last_loaded_script_name: None,
        })
    }

    pub fn map_lua<T>(&mut self, result: LuaResult<T>) -> Result<T, InteropError> {
        result.map_err(|e| lua_to_interop(&mut self.lua, e))
    }
}

fn lua_to_interop(lua: &mut Lua, e: LuaError) -> InteropError {
    InteropError::external(lua.get_error_message(e))
}

fn open_host_stdlibs(lua: &mut Lua) -> LuaResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        lua.open_stdlibs(&[
            Stdlib::Basic,
            Stdlib::Math,
            Stdlib::String,
            Stdlib::Table,
            Stdlib::Utf8,
            Stdlib::Coroutine,
            Stdlib::Package,
            Stdlib::Debug,
        ])
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        lua.open_stdlib(Stdlib::All)
    }
}

impl IntoScriptPluginParams for LuarsScriptingPlugin {
    type C = LuarsContext;
    type R = ();
    const LANGUAGE: Language = Language::Lua;

    fn build_runtime() -> Self::R {}

    fn handler() -> bevy_mod_scripting_core::handler::HandlerFn<Self> {
        luars_handler
    }

    fn context_loader() -> bevy_mod_scripting_core::context::ContextLoadFn<Self> {
        luars_context_load
    }

    fn context_reloader() -> bevy_mod_scripting_core::context::ContextReloadFn<Self> {
        luars_context_reload
    }
}

impl AsMut<ScriptingPlugin<Self>> for LuarsScriptingPlugin {
    fn as_mut(&mut self) -> &mut ScriptingPlugin<LuarsScriptingPlugin> {
        &mut self.scripting_plugin
    }
}

/// BMS language plugin backed by luars instead of mlua.
pub struct LuarsScriptingPlugin {
    pub scripting_plugin: ScriptingPlugin<Self>,
}

fn register_plugin_globals(context: &mut LuarsContext) -> Result<(), InteropError> {
    let register = context
        .lua
        .global_state_mut()
        .create_closure(|state| {
            let callback: String = state
                .get_arg_as(1)?
                .ok_or_else(|| state.error("register_callback expects a name".to_owned()))?;
            let func: LuaFunction = state
                .get_arg_as(2)?
                .ok_or_else(|| state.error("register_callback expects a function".to_owned()))?;

            let thread_ctxt = ThreadWorldContainer
                .try_get_context()
                .map_err(|e| state.error(format!("{e:?}")))?;
            let world = thread_ctxt.world;
            let attachment = world.current_attachment().0.ok_or_else(|| {
                state.error(
                    "Cannot register callback, missing script attachment context.".to_owned(),
                )
            })?;

            world
                .with_resource_mut(|res: Mut<ScriptCallbacks<LuarsScriptingPlugin>>| {
                    let mut callbacks = res.callbacks.write();
                    callbacks.insert(
                        (attachment.clone(), callback),
                        Arc::new(
                            move |args: Vec<ScriptValue>,
                                  lua: &mut LuarsContext,
                                  world_id: WorldId| {
                                let pre_handling_callbacks =
                                    LuarsScriptingPlugin::readonly_configuration(world_id)
                                        .pre_handling_callbacks;
                                pre_handling_callbacks
                                    .iter()
                                    .try_for_each(|init| init(&attachment, lua))?;

                                let mapped: Vec<LuaScriptValue> =
                                    args.into_iter().map(LuaScriptValue).collect();
                                func.call::<_, crate::script_value::MultiLuaScriptValue>(mapped)
                                    .map_err(|e| lua_to_interop(&mut lua.lua, e))
                                    .map(
                                        crate::script_value::MultiLuaScriptValue::into_script_value,
                                    )
                            },
                        ),
                    );
                })
                .map_err(|e| state.error(e.to_string()))?;
            Ok(0)
        })
        .map_err(|e| lua_to_interop(&mut context.lua, e))?;
    context
        .lua
        .set_global("register_callback", register)
        .map_err(|e| lua_to_interop(&mut context.lua, e))?;
    Ok(())
}

impl Default for LuarsScriptingPlugin {
    fn default() -> Self {
        LuarsScriptingPlugin {
            scripting_plugin: ScriptingPlugin {
                runtime_initializers: Vec::default(),
                supported_extensions: vec!["lua", "luau"],
                context_initializers: vec![
                    |_script_id, context| {
                        context
                            .lua
                            .set_global("world", LuaStaticReflectReference(TypeId::of::<World>()))
                            .map_err(|e| lua_to_interop(&mut context.lua, e))?;
                        register_plugin_globals(context)?;
                        Ok(())
                    },
                    |_script_id, context| {
                        let world = ThreadWorldContainer.try_get_context()?.world;
                        let globals_registry =
                            world.with_resource(|r: &AppScriptGlobalsRegistry| r.clone())?;
                        let globals_registry = globals_registry.read();

                        for (key, global) in globals_registry.iter() {
                            match &global.maker {
                                Some(maker) => {
                                    let global = (maker)(world.clone())?;
                                    context
                                        .lua
                                        .set_global(
                                            key.to_string().as_str(),
                                            LuaScriptValue::from(global),
                                        )
                                        .map_err(|e| lua_to_interop(&mut context.lua, e))?;
                                }
                                None => {
                                    let ref_ = LuaStaticReflectReference(global.type_id);
                                    context
                                        .lua
                                        .set_global(key.to_string().as_str(), ref_)
                                        .map_err(|e| lua_to_interop(&mut context.lua, e))?;
                                }
                            }
                        }

                        let script_function_registry = world.script_function_registry();
                        let script_function_registry = script_function_registry.read();

                        for (key, function) in script_function_registry
                            .iter_all()
                            .filter(|(k, _)| k.namespace == Namespace::Global)
                        {
                            context
                                .lua
                                .set_global(
                                    key.name.to_string().as_str(),
                                    LuaScriptValue::from(ScriptValue::Function(function.clone())),
                                )
                                .map_err(|e| lua_to_interop(&mut context.lua, e))?;
                        }

                        Ok(())
                    },
                ],
                context_pre_handling_initializers: vec![|context_key, context| {
                    let world = ThreadWorldContainer.try_get_context()?.world;
                    if let Some(entity) = context_key.entity() {
                        context
                            .lua
                            .set_global(
                                "entity",
                                LuaReflectReference(<Entity>::allocate(
                                    Box::new(entity),
                                    world.clone(),
                                )),
                            )
                            .map_err(|e| lua_to_interop(&mut context.lua, e))?;
                    }
                    context
                        .lua
                        .set_global(
                            "script_asset",
                            LuaReflectReference(<Handle<ScriptAsset>>::allocate(
                                Box::new(context_key.script()),
                                world,
                            )),
                        )
                        .map_err(|e| lua_to_interop(&mut context.lua, e))?;
                    Ok(())
                }],
                language: Language::Lua,
                context_policy: ContextPolicy::shared(),
                emit_responses: false,
                processing_pipeline_plugin: Default::default(),
            },
        }
    }
}

impl Plugin for LuarsScriptingPlugin {
    fn build(&self, app: &mut App) {
        self.scripting_plugin.build(app);
    }

    fn finish(&self, app: &mut App) {
        self.scripting_plugin.finish(app);
    }
}

fn load_lua_content_into_context(
    context: &mut LuarsContext,
    context_key: &ScriptAttachment,
    content: &[u8],
    world_id: WorldId,
) -> Result<(), InteropError> {
    let config = LuarsScriptingPlugin::readonly_configuration(world_id);
    let initializers = config.context_initialization_callbacks;
    let pre_handling_initializers = config.pre_handling_callbacks;
    initializers
        .iter()
        .try_for_each(|init| init(context_key, context))?;
    pre_handling_initializers
        .iter()
        .try_for_each(|init| init(context_key, context))?;

    let source = str::from_utf8(content).map_err(InteropError::external)?;
    context
        .lua
        .load(source)
        .set_name(
            context_key
                .script()
                .path()
                .map(|p| p.to_string())
                .unwrap_or_else(|| "script".to_owned()),
        )
        .exec()
        .map_err(|e| lua_to_interop(&mut context.lua, e))?;
    Ok(())
}

fn luars_context_load(
    context_key: &ScriptAttachment,
    content: &[u8],
    world_id: WorldId,
) -> Result<LuarsContext, InteropError> {
    let mut context = LuarsContext::new()?;
    context.last_loaded_script_name = context_key.script().path().map(|p| p.to_string());
    load_lua_content_into_context(&mut context, context_key, content, world_id)?;
    Ok(context)
}

fn luars_context_reload(
    context_key: &ScriptAttachment,
    content: &[u8],
    old_ctxt: &mut LuarsContext,
    world_id: WorldId,
) -> Result<(), InteropError> {
    load_lua_content_into_context(old_ctxt, context_key, content, world_id)?;
    Ok(())
}

fn luars_handler(
    args: Vec<ScriptValue>,
    context_key: &ScriptAttachment,
    callback_label: &CallbackLabel,
    context: &mut LuarsContext,
    world_id: WorldId,
) -> Result<ScriptValue, InteropError> {
    let config = LuarsScriptingPlugin::readonly_configuration(world_id);
    config
        .pre_handling_callbacks
        .iter()
        .try_for_each(|init| init(context_key, context))?;

    let handler: Option<LuaFunction> = context
        .lua
        .get_global(callback_label.as_ref())
        .map_err(|e| lua_to_interop(&mut context.lua, e))?;
    let Some(handler) = handler else {
        trace!(
            "Context {} is not subscribed to callback {}",
            context_key,
            callback_label.as_ref()
        );
        return Ok(ScriptValue::Unit);
    };

    let mapped: Vec<LuaScriptValue> = args.into_iter().map(LuaScriptValue).collect();
    handler
        .call::<_, MultiLuaScriptValue>(mapped)
        .map(MultiLuaScriptValue::into_script_value)
        .map_err(|e| lua_to_interop(&mut context.lua, e))
}

/// Convert a luars error into a BMS [`InteropError`] using the VM's stored message.
pub fn into_bms_error(lua: &mut Lua, e: LuaError) -> InteropError {
    lua_to_interop(lua, e)
}
