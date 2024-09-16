use std::sync::Mutex;
use bevy::app::AppExit;

use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;
use nano_9::{*, api::N9Args};
use crate::mlua::Variadic;
// use bevy_mod_scripting::api::lua;

// #[derive(LuaProxy, Reflect, Resource, Debug, Default)]
// #[reflect(Resource, LuaProxyable)]

// #[proxy(
//     derive(clone),
//     functions[
//         r#"
//         #[lua(kind="MutatingMethod")]
//         fn set_my_string(&mut self, another_string: Option<String>);
//         "#,
//         r#"
//         #[lua(kind="MutatingMethod")]
//         fn set_with_another(&mut self, #[proxy] another: Self);
//         "#,
//         r#"
//         #[lua(kind="Method")]
//         fn get_my_string(&self) -> String;
//         "#,
//         r#"
//         #[lua(kind="Method",raw)]
//         fn raw_method(&self, ctx : &Lua) -> Result<String, _> {
//             let a = ctx.globals().get::<_,String>("world").unwrap();
//             let a = self.inner()?;
//             Ok("".to_owned())
//         }
//         "#,
//         r#"
//         #[lua(kind="MetaMethod", metamethod="ToString")]
//         fn to_string(&self) -> String {
//             format!("{:#?}", _self)
//         }
//         "#
//     ])
//     ]
// pub struct Out {
//     result: bool
// }

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

#[test]
fn change_camera_position() {
    let mut app = App::new();
    app
        .add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(bevy::asset::AssetPlugin::default())
        .add_plugins(bevy::render::prelude::ImagePlugin::default())
        // .add_plugins(bevy::render::RenderPlugin::default())
        // .add_event::<bevy::window::RequestRedraw>()
        // .add_plugins(DefaultPlugins)
        .add_plugins(Nano9Plugin::default())
        .add_api_provider::<LuaScriptHost<N9Args>>(Box::new(TestAPI))
        // .register_type::<Failed>()
        // .init_resource::<Failed>()
        .add_systems(Startup,
            |world: &mut World| {

                let entity = world.spawn(()).id();

                // run script
                world.resource_scope(|world, mut host: Mut<LuaScriptHost<N9Args>>| {
                    host.run_one_shot(
                        r#"
                        function once()
                            local Out = world:get_type_by_name("Out")
                            local out = world:get_resource(Out);
                            camera.x = 1
                            if camera.x ~= 1 then
                                fail("camera x not set.");
                            end
                        end
                        "#
                        .as_bytes(),
                        "script.lua",
                        entity,
                        world,
                        LuaEvent {
                            hook_name: "once".to_owned(),
                            args: Variadic::new(),
                            recipients: Recipients::All,
                        },
                    )
                    .expect("Something went wrong in the script!");
                });

            },
        );

    app.update();
    if let Some(failed) = app.world().get_resource::<Failed>() {
        if let Some(msg) = &failed.0 {
            assert!(false, "{}", msg);
        } else {
            assert!(false);
        }
    }
    // assert!(out.result == Some(false));
    // assert!(out.result == None)
}
