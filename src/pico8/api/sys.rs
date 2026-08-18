use super::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use std::path::{Path, PathBuf};

pub(crate) fn plugin(app: &mut App) {
    app.init_resource::<ExtcmdState>();
    #[cfg(feature = "scripting")]
    lua::plugin(app);
}

/// State for Pico-8 `extcmd` (screenshots and shutdown).
#[derive(Resource, Debug)]
pub struct ExtcmdState {
    /// Stem for the next screenshot (`extcmd("set_filename", name)`).
    pub filename: Option<String>,
    /// Directory to write screenshots into.
    pub screenshot_dir: PathBuf,
    /// A GPU screenshot is in flight.
    pub capturing: bool,
    /// Call `extcmd("shutdown")` while a screenshot is capturing.
    pub shutdown_after: bool,
}

impl Default for ExtcmdState {
    fn default() -> Self {
        Self {
            filename: None,
            screenshot_dir: std::env::var_os("NANO9_SCREENSHOT_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            capturing: false,
            shutdown_after: false,
        }
    }
}

impl super::Pico8<'_, '_> {
    pub fn time(&self) -> f32 {
        self.time.elapsed_secs()
    }

    pub fn delta_time(&self) -> f32 {
        self.time.delta_secs()
    }

    pub fn exit(&mut self, error: Option<u8>) {
        self.commands
            .write_message(match error.and_then(std::num::NonZero::new) {
                Some(n) => AppExit::Error(n),
                None => AppExit::Success,
            });
    }

    /// Pico-8 `extcmd(cmd, [p1], [p2])`.
    ///
    /// Supported: `set_filename`, `screen`, `shutdown`.
    pub fn extcmd(&mut self, cmd: &str, p1: Option<&str>, p2: Option<f32>) -> Result<(), Error> {
        match cmd {
            "set_filename" => {
                if let Some(name) = p1 {
                    self.extcmd.filename = Some(name.to_string());
                }
                Ok(())
            }
            "screen" => {
                if self.extcmd.capturing || self.extcmd.shutdown_after {
                    return Ok(());
                }
                let _scale = p1.and_then(|s| s.parse::<f32>().ok()).or(p2);
                let _save_to_folder = p2;
                self.request_screenshot()
            }
            "shutdown" => {
                if self.extcmd.capturing {
                    self.extcmd.shutdown_after = true;
                } else {
                    self.exit(None);
                }
                Ok(())
            }
            other => {
                warn!("extcmd({other:?}) is not implemented");
                Ok(())
            }
        }
    }

    fn screenshot_path(&self) -> PathBuf {
        let stem = self
            .extcmd
            .filename
            .clone()
            .unwrap_or_else(|| "nano9".to_string());
        let mut path = self.extcmd.screenshot_dir.clone();
        path.push(format!("{stem}.png"));
        path
    }

    fn request_screenshot(&mut self) -> Result<(), Error> {
        let path = self.screenshot_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let canvas_size = self.canvas.size;
        self.extcmd.capturing = true;
        self.commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>,
                  cameras: Query<&Camera, With<Nano9Camera>>,
                  mut commands: Commands,
                  mut extcmd: ResMut<ExtcmdState>| {
                if let Err(e) =
                    save_captured_screenshot(&captured.image, cameras, canvas_size, &path)
                {
                    error!("extcmd(\"screen\") failed: {e}");
                } else {
                    info!("Screenshot saved to {}", path.display());
                }
                extcmd.capturing = false;
                if extcmd.shutdown_after {
                    commands.write_message(AppExit::Success);
                }
            },
        );
        Ok(())
    }
}

fn save_captured_screenshot(
    image: &Image,
    cameras: Query<&Camera, With<Nano9Camera>>,
    canvas_size: UVec2,
    path: &Path,
) -> Result<(), String> {
    let src_size = image.size();
    let data = image
        .data
        .as_ref()
        .ok_or_else(|| "screenshot image has no CPU data".to_string())?;
    let bpp = 4usize;
    if data.len() < src_size.x as usize * src_size.y as usize * bpp {
        return Err(format!(
            "screenshot buffer too small: {} bytes for {}x{}",
            data.len(),
            src_size.x,
            src_size.y
        ));
    }
    let bgra = matches!(
        image.texture_descriptor.format,
        TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb
    );

    let crop = cameras
        .iter()
        .find_map(|camera| {
            camera.viewport.as_ref().map(|vp| {
                (
                    vp.physical_position.x.min(src_size.x.saturating_sub(1)),
                    vp.physical_position.y.min(src_size.y.saturating_sub(1)),
                    vp.physical_size.x.max(1).min(src_size.x),
                    vp.physical_size.y.max(1).min(src_size.y),
                )
            })
        })
        .unwrap_or((0, 0, src_size.x, src_size.y));

    let (cx, cy, cw, ch) = crop;
    let cw = cw.min(src_size.x.saturating_sub(cx)).max(1);
    let ch = ch.min(src_size.y.saturating_sub(cy)).max(1);
    let dst_w = canvas_size.x.max(1);
    let dst_h = canvas_size.y.max(1);

    let mut rgb = vec![0u8; dst_w as usize * dst_h as usize * 3];
    for y in 0..dst_h {
        let sy = cy + y * ch / dst_h;
        for x in 0..dst_w {
            let sx = cx + x * cw / dst_w;
            let si = ((sy * src_size.x + sx) as usize) * bpp;
            let di = ((y * dst_w + x) as usize) * 3;
            if bgra {
                rgb[di] = data[si + 2];
                rgb[di + 1] = data[si + 1];
                rgb[di + 2] = data[si];
            } else {
                rgb[di] = data[si];
                rgb[di + 1] = data[si + 1];
                rgb[di + 2] = data[si + 2];
            }
        }
    }
    write_rgb_png(path, dst_w, dst_h, &rgb)
}

fn write_rgb_png(path: &Path, width: u32, height: u32, rgb: &[u8]) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgb).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(feature = "scripting")]
mod lua {
    use super::*;
    use crate::pico8::lua::with_pico8;

    use bevy_mod_scripting::bindings::ScriptValue;
    use bevy_mod_scripting::bindings::function::{
        namespace::{GlobalNamespace, NamespaceBuilder},
        script_function::FunctionCallContext,
    };

    pub(crate) fn plugin(app: &mut App) {
        let world = app.world_mut();
        NamespaceBuilder::<GlobalNamespace>::new_unregistered(world)
            .register("exit", |ctx: FunctionCallContext, error: Option<u8>| {
                with_pico8(&ctx, move |pico8| {
                    pico8.exit(error);
                    Ok(())
                })
            })
            .register("time", |ctx: FunctionCallContext| {
                with_pico8(&ctx, move |pico8| Ok(pico8.time()))
            })
            .register("delta_time", |ctx: FunctionCallContext| {
                with_pico8(&ctx, move |pico8| Ok(pico8.delta_time()))
            })
            .register(
                "extcmd",
                |ctx: FunctionCallContext,
                 cmd: String,
                 p1: Option<ScriptValue>,
                 p2: Option<ScriptValue>| {
                    let p1s = p1.as_ref().and_then(script_value_to_string);
                    let p2n = p2.as_ref().and_then(script_value_to_f32);
                    with_pico8(&ctx, move |pico8| pico8.extcmd(&cmd, p1s.as_deref(), p2n))
                },
            );
    }

    fn script_value_to_string(value: &ScriptValue) -> Option<String> {
        match value {
            ScriptValue::String(s) => Some(s.to_string()),
            ScriptValue::Integer(i) => Some(i.to_string()),
            ScriptValue::Float(f) => Some(f.to_string()),
            _ => None,
        }
    }

    fn script_value_to_f32(value: &ScriptValue) -> Option<f32> {
        match value {
            ScriptValue::Integer(i) => Some(*i as f32),
            ScriptValue::Float(f) => Some(*f as f32),
            ScriptValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
}
