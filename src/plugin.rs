#![allow(deprecated)]

use bevy::{
    prelude::*,
    reflect::Reflect,
    render::{
        camera::ScalingMode,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    utils::Duration,
    window::PresentMode,
    window::{PrimaryWindow, WindowResized, WindowResolution},
};

use bevy_asset_loader::prelude::*;
use bevy_mod_scripting::{
    prelude::*,
    core::event::ScriptLoaded,
};
// use bevy_pixel_buffer::prelude::*;
use crate::{
    assets::{ImageHandles},
    screens,
};

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "images/pico-8-palette.png")]
    palette: Handle<Image>,
}

#[derive(Component)]
pub struct Nano9Sprite;

#[derive(Resource)]
pub struct Nano9SpriteSheet(pub Handle<Image>, pub Handle<TextureAtlasLayout>);

#[derive(Resource)]
pub struct Nano9Screen(pub Handle<Image>);

#[derive(Resource, Default)]
pub struct DrawState {
    pub pen: Color,
    pub camera_position: Vec2,
    pub print_cursor: Vec2,
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
            display_grid_dimensions: (512, 512),
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
    let mut image = Image::new_fill(
        Extent3d {
            width: settings.physical_grid_dimensions.0,
            height: settings.physical_grid_dimensions.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0u8, 0u8, 0u8, 255u8],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    let handle = assets.add(image);
    commands.insert_resource(Nano9Screen(handle.clone()));
    let mut camera_bundle = Camera2dBundle::default();
    // camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(512.0);
    camera_bundle.projection.scaling_mode = ScalingMode::WindowSize(8.0);

    commands.spawn(camera_bundle);
    commands
        .spawn(SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, -1.0),
            texture: handle,
            sprite: Sprite {
                // custom_size: Some(Vec2::new(
                //     settings.display_grid_dimensions.0 as f32,
                //     settings.display_grid_dimensions.1 as f32,
                // )),
                ..default()
            },
            ..default()
        })
        .insert(Nano9Sprite);
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
    eprintln!("init");
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
        app.insert_resource(bevy::winit::WinitSettings {
            // focused_mode: bevy::winit::UpdateMode::Continuous,
            focused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                wait: Duration::from_millis(16),
            },
            unfocused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                wait: Duration::from_millis(16 * 4),
            },
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(512.0, 512.0), //.with_scale_factor_override(1.0),
                        // Turn off vsync to maximize CPU/GPU usage
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(Time::<Fixed>::from_seconds(UPDATE_FREQUENCY.into()))
        .init_resource::<Settings>()
        .init_resource::<DrawState>()
        .add_plugins(ScriptingPlugin)
        .add_plugins(crate::plugin)
        .add_systems(OnExit(screens::Screen::Loading), setup_image)
        // .add_systems(OnEnter(screens::Screen::Playing), send_init)
        // .add_systems(PreUpdate, send_init.run_if(on_asset_modified::<LuaFile>()))
        .add_systems(PreUpdate, send_init.run_if(on_event::<ScriptLoaded>()))
        // .add_systems(Update, sync_window_size)
        .add_systems(
            FixedUpdate,
            (send_update, send_draw)
                .chain()
                .run_if(in_state(screens::Screen::Playing)),
        );
    }
}
pub fn on_asset_modified<T: Asset>() -> impl FnMut(EventReader<AssetEvent<T>>) -> bool + Clone {
    // The events need to be consumed, so that there are no false positives on subsequent
    // calls of the run condition. Simply checking `is_empty` would not be enough.
    // PERF: note that `count` is efficient (not actually looping/iterating),
    // due to Bevy having a specialized implementation for events.
    move |mut reader: EventReader<AssetEvent<T>>| reader.read().any(|e| matches!(e, AssetEvent::Modified { .. }))
}
