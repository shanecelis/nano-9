use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;
use nano_9::*;

fn main() -> std::io::Result<()> {
    App::new()
        .add_plugins(Nano9Plugin)
        .add_systems(Startup, setup)
        .run();

    Ok(())
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    let script_path = "scripts/basic.lua";

    commands.spawn(ScriptCollection::<LuaFile> {
        scripts: vec![Script::new(
            script_path.to_owned(),
            asset_server.load(script_path),
        )],
    });
}
