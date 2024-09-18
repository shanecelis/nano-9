use std::sync::Mutex;
use bevy::app::AppExit;

use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;
use nano_9::{*, api::N9Args};
use crate::mlua::Variadic;

#[derive(Resource, Default)]
pub struct Failed(Option<String>);

pub struct TestAPI;

impl APIProvider for TestAPI {
    type APITarget = Mutex<Lua>;
    type ScriptContext = Mutex<Lua>;
    type DocTarget = LuaDocFragment;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        let ctx = ctx.get_mut().unwrap();
        ctx.globals()
            .set(
                "fail",
                ctx.create_function(|ctx, msg: Option<String>| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    world.insert_resource(Failed(msg));
                    Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;
        Ok(())
    }
}

fn new_app() -> App {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(bevy::render::prelude::ImagePlugin::default())
        .add_plugins(Nano9Plugin::default())
        .add_api_provider::<LuaScriptHost<N9Args>>(Box::new(TestAPI));
    app
}

fn run_lua_test(script: impl Into<String>) {
    let mut app = new_app();
    let script = script.into();
    app
        .add_systems(Update,
            move |world: &mut World| {

                let entity = world.spawn(()).id();

                // run script
                world.resource_scope(|world, mut host: Mut<LuaScriptHost<N9Args>>| {
                    if let Err(e) = host.run_one_shot(
                        script.as_bytes(),
                        "script.lua",
                        entity,
                        world,
                        LuaEvent {
                            hook_name: "once".to_owned(),
                            args: Variadic::new(),
                            recipients: Recipients::All,
                        },
                    ) {
                        panic!("{}", e);
                    }
                    // .expect("Something went wrong in the script!");
                });

            },
        );

    app.update();
    if let Some(events) = app.world().get_resource::<Events<ScriptErrorEvent>>() {
        let mut reader = events.get_reader();
        for r in reader.read(&events) {
            assert!(false, "{}", r.error);
        }

    }

    if let Some(failed) = app.world().get_resource::<Failed>() {
        if let Some(msg) = &failed.0 {
            assert!(false, "{}", msg);
        } else {
            assert!(false);
        }
    }
}

#[test]
fn change_camera_position() {
    run_lua_test(
        r#"
        function once()
            camera.x = 1
            if camera.x ~= 1 then
                fail("camera x not set.");
            end
        end
        "#);
}

#[test]
fn default_camera_position() {
    run_lua_test(
        r#"
        function once()
            --notthere.y
            if camera.x ~= 64 then

                fail("camera set to "..camera.x);
                --fail("camera set to ");
            end
        end
        "#);
}

#[test]
fn render_text() {
    run_lua_test(
        r#"
        function once()
            text:print("Hello, World!")
        end
        "#);
}

// #[test]
// fn test_fail() {
//     run_lua_test(
//         r#"
//         function once()
//             fail("what")
//         end
//         "#);
// }
