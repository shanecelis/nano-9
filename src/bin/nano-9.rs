use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;
use nano_9::*;
use std::env;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() -> std::io::Result<()> {
    let mut args = env::args();
    let script_path: String = args
        .skip(1)
        .next()
        .map(|s| format!("../{s}"))
        .unwrap_or("scripts/main.lua".into());
    App::new()
        .add_plugins(Nano9Plugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(
            Startup,
            move |asset_server: Res<AssetServer>, mut commands: Commands| {
                commands.spawn(ScriptCollection::<LuaFile> {
                    scripts: vec![Script::new(
                        script_path.clone(),
                        asset_server.load(&script_path),
                    )],
                });
            },
        )
        .run();
    Ok(())
}
