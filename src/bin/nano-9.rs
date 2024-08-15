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

fn setup(asset_server: Res<AssetServer>, mut commands: Commands,
         mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let script_path = "scripts/basic.lua";
    // let image = asset_server.load("images/oh no more goblins.png");
    // let layout = TextureAtlasLayout::from_grid(Vec2::splat(12.0), 24, 24, None, None);
    // let image = asset_server.load("images/Cat_Sprite.png");
    // let layout = TextureAtlasLayout::from_grid(Vec2::splat(32.0), 4, 8, None, None);
    // commands.insert_resource(Nano9SpriteSheet(image, texture_atlas_layouts.add(layout)));

    commands.spawn(ScriptCollection::<LuaFile> {
        scripts: vec![Script::new(
            script_path.to_owned(),
            asset_server.load(script_path),
        )],
    });
}
