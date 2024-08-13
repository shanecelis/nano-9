#![allow(deprecated)]
use std::sync::Mutex;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    window::PresentMode,
    ecs::system::SystemState,
    prelude::*,
    reflect::Reflect,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    window::{PrimaryWindow, WindowResized, WindowResolution},
    utils::Duration,
};

use bevy_mod_scripting::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_pixel_buffer::prelude::*;
use crate::{screens, assets::{self, ImageHandles}};

#[derive(AssetCollection, Resource)]
struct ImageAssets {
  #[asset(path = "images/pico-8-palette.png")]
  palette: Handle<Image>,
  // #[asset(key = "tree")]
  // tree: Handle<Image>,
}

// #[derive(Debug, Default, Clone, Reflect, Component, LuaProxy)]
// #[reflect(Component, LuaProxyable)]
// pub struct LifeState {
//     pub cells: Vec<u8>,
// }

#[derive(Default)]
pub struct Nano9API;

#[derive(Component)]
pub struct Nano9Sprite;

#[derive(Resource)]
pub struct Nano9Palette(Handle<Image>);

#[derive(Resource, Default)]
pub struct DrawState {
    pen: Color,
    camera_position: Vec2,
    print_cursor: Vec2,
    // palette_modifications:
}

impl APIProvider for Nano9API {
    type APITarget = Mutex<Lua>;
    type ScriptContext = Mutex<Lua>;
    type DocTarget = LuaDocFragment;

    fn attach_api(&mut self, ctx: &mut Self::APITarget) -> Result<(), ScriptError> {
        // callbacks can receive any `ToLuaMulti` arguments, here '()' and
        // return any `FromLuaMulti` arguments, here a `usize`
        // check the Rlua documentation for more details

        let ctx = ctx.get_mut().unwrap();

        ctx.globals()
            .set(
                "pset",
                ctx.create_function(|ctx, (x, y, c): (f32, f32, Value)| {
                    let world = ctx.get_world()?;
                    let mut world = world.write();
                    let mut system_state: SystemState<(Query<PixelBuffers>, Res<Nano9Palette>, ResMut<Assets<Image>>)> = SystemState::new(&mut world);
                    let (pixel_buffers, palette, mut images) = system_state.get_mut(&mut world);
                    let color = match c {
                        Value::Integer(n) => {
                            let pal = images.get_mut(&palette.0).unwrap();
                            dbg!(&pal.texture_descriptor);
                            // pal.data
                            //
                            let frame = pal.frame();
                            // frame.get(UVec2::new(n as u32, 0));
                            frame.raw()[n as usize]
                        }
                        _ => todo!()
                    };

                    for item in pixel_buffers.iter() {
                        images.frame(item).set((x as u32, y as u32), color);
                    // let mut frame = query_pixel_buffer.frame();
                    // frame.set((x as u32, y as u32), Pixel::RED);
                    }
                    Ok(())
                })
                .map_err(ScriptError::new_other)?,
            )
            .map_err(ScriptError::new_other)?;

        Ok(())
    }

    fn register_with_app(&self, app: &mut App) {
        // this will register the `LuaProxyable` typedata since we derived it
        // this will resolve retrievals of this component to our custom lua object
        // app.register_type::<LifeState>();
        app.register_type::<Settings>();
    }
}

#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct Settings {
    // TODO: Change to UVec2
    physical_grid_dimensions: (u32, u32),
    display_grid_dimensions: (u32, u32),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            physical_grid_dimensions: (128, 128),
            display_grid_dimensions: (0, 0),
        }
    }
}

pub fn setup_image(
    mut commands: Commands,
    image_handles: Res<ImageHandles>,
    mut assets: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
) {
    // let mut image = Image::new_fill(
    //     Extent3d {
    //         width: settings.physical_grid_dimensions.0,
    //         height: settings.physical_grid_dimensions.1,
    //         depth_or_array_layers: 1,
    //     },
    //     TextureDimension::D2,
    //     &[0u8, 0u8, 0u8, 255u8],
    //     TextureFormat::Rgba8Unorm,
    //     RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    // );

    // image.sampler = ImageSampler::nearest();
    // let handle = asset_server.load("images/pico-8-palette.png");
    // let mut image = assets.get(image_handles.get(ImageHandles::PICO8_PALETTE).unwrap())
    //                       .expect("image")
    //                       .convert(Pixel::FORMAT)
    //                       .expect("convert to pixel format");
    // image.sampler = ImageSampler::nearest();

    // commands.insert_resource(Nano9Palette(assets.add(image)));
    commands.insert_resource(Nano9Palette(image_handles.get(ImageHandles::PICO8_PALETTE).unwrap().clone()));

    // let script_path = bevy_mod_scripting_lua::lua_path!("game_of_life");
    // let handle = assets.add(image);
    // commands.insert_resource(Nano9Image(handle.clone()));
    // commands.spawn(Camera2dBundle::default());
    // commands
    //     .spawn(SpriteBundle {
    //         texture: handle,
    //         sprite: Sprite {
    //             custom_size: Some(Vec2::new(
    //                 settings.display_grid_dimensions.0 as f32,
    //                 settings.display_grid_dimensions.1 as f32,
    //             )),
    //             color: Color::TOMATO,
    //             ..Default::default()
    //         },
    //         ..Default::default()
    //     })
    //     .insert(Nano9Sprite);
        // .insert(LifeState {
        //     cells: vec![
        //         0u8;
        //         (settings.physical_grid_dimensions.0 * settings.physical_grid_dimensions.1)
        //             as usize
        //     ],
        // })
        // .insert(ScriptCollection::<LuaFile> {
        //     scripts: vec![Script::new(
        //         script_path.to_owned(),
        //         asset_server.load(script_path),
        //     )],
        // });
}

pub fn sync_window_size(
    mut resize_event: EventReader<WindowResized>,
    mut settings: ResMut<Settings>,
    mut query: Query<&mut Sprite, With<Nano9Sprite>>,
    primary_windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(e) = resize_event
        .read()
        .filter(|e| primary_windows.get(e.window).is_ok())
        .last()
    {
        let primary_window = primary_windows.get(e.window).unwrap();
        settings.display_grid_dimensions = (
            primary_window.physical_width(),
            primary_window.physical_height(),
        );

        // resize all game's of life, retain aspect ratio and fit the entire game in the window
        for mut sprite in query.iter_mut() {
            let scale = if settings.physical_grid_dimensions.0 > settings.physical_grid_dimensions.1
            {
                // horizontal is longer
                settings.display_grid_dimensions.1 as f32
                    / settings.physical_grid_dimensions.1 as f32
            } else {
                // vertical is longer
                settings.display_grid_dimensions.0 as f32
                    / settings.physical_grid_dimensions.0 as f32
            };

            sprite.custom_size = Some(Vec2::new(
                (settings.physical_grid_dimensions.0 as f32) * scale,
                (settings.physical_grid_dimensions.1 as f32) * scale,
            ));
        }
    }
}

/// Runs after LifeState components are updated, updates their rendered representation
// pub fn update_rendered_state(
//     mut assets: ResMut<Assets<Image>>,
//     query: Query<(&LifeState, &Handle<Image>)>,
// ) {
//     for (new_state, old_rendered_state) in query.iter() {
//         let old_rendered_state = assets
//             .get_mut(old_rendered_state)
//             .expect("World is not setup correctly");

//         old_rendered_state.data = new_state.cells.clone();
//     }
// }

/// Sends events allowing scripts to drive update logic
pub fn send_update(mut events: PriorityEventWriter<LuaEvent<()>>) {
    events.send(
        LuaEvent {
            hook_name: "_update".to_owned(),
            args: (),
            recipients: Recipients::All,
        },
        1,
    )
}

/// Sends initialization event
pub fn send_init(mut events: PriorityEventWriter<LuaEvent<()>>) {
    events.send(
        LuaEvent {
            hook_name: "_init".to_owned(),
            args: (),
            recipients: Recipients::All,
        },
        0,
    )
}

/// Sends initialization event
pub fn send_draw(mut events: PriorityEventWriter<LuaEvent<()>>) {
    events.send(
        LuaEvent {
            hook_name: "_draw".to_owned(),
            args: (),
            recipients: Recipients::All,
        },
        0,
    )
}

const UPDATE_FREQUENCY: f32 = 1.0 / 60.0;

pub struct Nano9Plugin;

impl Plugin for Nano9Plugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(bevy::winit::WinitSettings {
                // focused_mode: bevy::winit::UpdateMode::Continuous,
                focused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                    wait: Duration::from_millis(16),
                },
                unfocused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                    wait: Duration::from_millis(16),
                },
            })
            .add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: WindowResolution::new(512.0, 512.0),//.with_scale_factor_override(1.0),
                    // Turn off vsync to maximize CPU/GPU usage
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }))
            .insert_resource(Time::<Fixed>::from_seconds(UPDATE_FREQUENCY.into()))
            .init_resource::<Settings>()
            // .add_plugins(LogDiagnosticsPlugin::default())
            // .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(PixelBufferPlugin)
            .add_plugins(ScriptingPlugin)

            .add_plugins((
                // demo::plugin,
                screens::plugin,
                // theme::plugin,
                assets::plugin,
                // audio::plugin,
            ))
            .add_systems(
                Startup,
                PixelBufferBuilder::new()
                    .with_size(PixelBufferSize {
                        size: UVec2::new(128, 128),
                        pixel_size: UVec2::new(4, 4)
                    })
                    // .with_fill(Fill::window())//.with_stretch(true)) // set fill to the window
                    .setup(),
            )
            .add_systems(OnExit(screens::Screen::Loading), setup_image)
            .add_systems(OnEnter(screens::Screen::Playing), send_init)
            // .add_systems(Update, sync_window_size)
            // .add_systems(Update, wild_update)
        // .add_systems(FixedUpdate, update_rendered_state.after(sync_window_size))
            .add_systems(FixedUpdate, (send_update, send_draw).chain().run_if(in_state(screens::Screen::Playing)))
            .add_systems(FixedUpdate, script_event_handler::<LuaScriptHost<()>, 0, 1>)
            .add_script_host::<LuaScriptHost<()>>(PostUpdate)
            .add_api_provider::<LuaScriptHost<()>>(Box::new(LuaCoreBevyAPIProvider))
            .add_api_provider::<LuaScriptHost<()>>(Box::new(Nano9API));
    }
}

fn wild_update(mut pb: QueryPixelBuffer) {
    pb.frame().per_pixel(|_, _| Pixel::random());
}
