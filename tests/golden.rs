//! Compare Nano-9 screenshots against Pico-8 goldens in `tests/golden/`.
//!
//! Generate goldens:
//! ```sh
//! bin/golden-pico8
//! ```
//!
//! Run (GPU window required). Some primitives mismatch; that is expected —
//! the point is to see the diff, not to paper over it:
//! ```sh
//! cargo test-golden
//! ```
//!
//! Carts are every `tests/golden/*.p8`. Screenshots live next to them as
//! `{name}-expected.png` (Pico-8) and `{name}-actual.png` (Nano-9). On
//! mismatch a `{name}-compare.png` is shown with `wezterm imgcat` when a
//! tty is available.

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn load_rgb_png(path: &Path) -> (u32, u32, Vec<u8>) {
    let file = File::open(path).unwrap_or_else(|e| panic!("open {}: {e}", path.display()));
    let decoder = png::Decoder::new(file);
    let mut reader = decoder
        .read_info()
        .unwrap_or_else(|e| panic!("png {}: {e}", path.display()));
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .unwrap_or_else(|e| panic!("frame {}: {e}", path.display()));
    let width = info.width;
    let height = info.height;
    let rgb = match info.color_type {
        png::ColorType::Rgb => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgba => buf[..info.buffer_size()]
            .chunks_exact(4)
            .flat_map(|p| [p[0], p[1], p[2]])
            .collect(),
        png::ColorType::Grayscale => buf[..info.buffer_size()]
            .iter()
            .flat_map(|g| [*g, *g, *g])
            .collect(),
        other => panic!("{}: unsupported color type {other:?}", path.display()),
    };
    (width, height, rgb)
}

fn write_rgb_png(path: &Path, width: u32, height: u32, rgb: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let file = File::create(path).unwrap_or_else(|e| panic!("create {}: {e}", path.display()));
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(rgb).unwrap();
}

fn make_diff(a: &[u8], b: &[u8]) -> (Vec<u8>, usize) {
    let n = a.len().min(b.len());
    let mut diff = vec![0u8; n];
    let mut changed = 0usize;
    for i in (0..n).step_by(3) {
        if a[i] != b[i] || a[i + 1] != b[i + 1] || a[i + 2] != b[i + 2] {
            changed += 1;
            diff[i] = 255;
            diff[i + 1] = 0;
            diff[i + 2] = 255;
        } else {
            diff[i] = a[i] / 3;
            diff[i + 1] = a[i + 1] / 3;
            diff[i + 2] = a[i + 2] / 3;
        }
    }
    (diff, changed)
}

fn put_pixel(out: &mut [u8], stride: u32, x: u32, y: u32, rgb: [u8; 3]) {
    let i = ((y * stride + x) * 3) as usize;
    out[i] = rgb[0];
    out[i + 1] = rgb[1];
    out[i + 2] = rgb[2];
}

/// Pico-8 | Nano-9 | magenta mismatches, nearest-neighbor scaled so pixels read.
fn compose_compare(
    expected: &[u8],
    actual: &[u8],
    diff: &[u8],
    w: u32,
    h: u32,
) -> (u32, u32, Vec<u8>) {
    const SCALE: u32 = 3;
    const GAP: u32 = 6;
    const BAR: u32 = 8;
    // Pico-8 green, Nano-9 orange, mismatch magenta.
    const BARS: [[u8; 3]; 3] = [[0, 231, 86], [255, 163, 0], [255, 0, 255]];
    let pw = w * SCALE;
    let ph = h * SCALE;
    let out_w = pw * 3 + GAP * 2;
    let out_h = BAR + ph;
    let mut out = vec![0u8; (out_w * out_h * 3) as usize];
    let panels = [expected, actual, diff];
    for (i, panel) in panels.iter().enumerate() {
        let ox = (pw + GAP) * i as u32;
        for y in 0..BAR {
            for x in 0..pw {
                put_pixel(&mut out, out_w, ox + x, y, BARS[i]);
            }
        }
        for y in 0..ph {
            let sy = y / SCALE;
            for x in 0..pw {
                let sx = x / SCALE;
                let si = ((sy * w + sx) * 3) as usize;
                put_pixel(
                    &mut out,
                    out_w,
                    ox + x,
                    BAR + y,
                    [panel[si], panel[si + 1], panel[si + 2]],
                );
            }
        }
    }
    (out_w, out_h, out)
}

fn open_tty() -> Option<File> {
    File::options().write(true).open("/dev/tty").ok()
}

fn base64_encode(data: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (a << 16) | (b << 8) | c;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(T[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(T[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Show a PNG in WezTerm. Writes to `/dev/tty` so `cargo test` capture cannot swallow it.
fn show_image(label: &str, path: &Path) {
    let Some(mut tty) = open_tty() else {
        return;
    };
    let _ = writeln!(tty, "{label}");
    let _ = tty.flush();
    let shown = Command::new("wezterm")
        .args(["imgcat", "--width", "80%", "--resample-filter", "nearest"])
        .arg(path)
        .stdout(tty)
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if shown {
        return;
    }
    // iTerm2 / WezTerm inline-image protocol, in case `wezterm` is not on PATH.
    let Ok(bytes) = fs::read(path) else {
        return;
    };
    if let Some(mut tty) = open_tty() {
        let b64 = base64_encode(&bytes);
        let _ = write!(
            tty,
            "\x1b]1337;File=inline=1;width=80%;height=auto;preserveAspectRatio=1:{b64}\x07\n"
        );
        let _ = tty.flush();
    }
}

fn run_n9(cart: &Path, actual_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(actual_dir).map_err(|e| e.to_string())?;
    let n9 = option_env!("CARGO_BIN_EXE_n9").unwrap_or("n9");
    let mut child = Command::new(n9)
        .args(["run", cart.to_str().unwrap()])
        .env("NANO9_SCREENSHOT_DIR", actual_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn n9: {e}"))?;

    // Drain stderr so Bevy logs cannot fill the pipe and deadlock n9.
    let stderr = child.stderr.take();
    let stderr_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(mut err) = stderr {
            use std::io::Read;
            let _ = err.read_to_string(&mut buf);
        }
        buf
    });

    let timeout = Duration::from_secs(20);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = stderr_thread.join().unwrap_or_default();
                if status.success() {
                    return Ok(());
                }
                return Err(format!("n9 exited {status}: {stderr}"));
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let stderr = stderr_thread.join().unwrap_or_default();
                    let tail: String = stderr
                        .chars()
                        .rev()
                        .take(1500)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    return Err(format!("n9 timed out after {timeout:?}: {tail}"));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("wait n9: {e}")),
        }
    }
}

enum CartResult {
    Match,
    Differ {
        changed: usize,
        total: usize,
        compare: PathBuf,
    },
}

fn cart_names() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(golden_dir())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "p8") {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

fn check_cart(name: &str) -> Result<CartResult, String> {
    let dir = golden_dir();
    let cart = dir.join(format!("{name}.p8"));
    let expected = dir.join(format!("{name}-expected.png"));
    let written = dir.join(format!("{name}.png"));
    let actual = dir.join(format!("{name}-actual.png"));

    if !cart.exists() {
        return Err(format!("missing cart {}", cart.display()));
    }
    if !expected.exists() {
        return Err(format!(
            "missing golden {} (run bin/golden-pico8)",
            expected.display()
        ));
    }

    let _ = fs::remove_file(&written);
    run_n9(&cart, &dir)?;
    if written.exists() {
        fs::rename(&written, &actual).map_err(|e| e.to_string())?;
    }
    if !actual.exists() {
        return Err(format!("n9 did not write {}", actual.display()));
    }

    let (ew, eh, ebytes) = load_rgb_png(&expected);
    let (aw, ah, abytes) = load_rgb_png(&actual);
    if (ew, eh) != (aw, ah) {
        println!("\n=== {name}: size Pico-8 {ew}x{eh} vs Nano-9 {aw}x{ah} ===");
        show_image("Pico-8", &expected);
        show_image("Nano-9", &actual);
        return Err(format!("size {ew}x{eh} vs {aw}x{ah}"));
    }
    if ebytes == abytes {
        return Ok(CartResult::Match);
    }

    let (diff, changed) = make_diff(&ebytes, &abytes);
    let diff_path = dir.join(format!("{name}-diff.png"));
    write_rgb_png(&diff_path, ew, eh, &diff);
    let (cw, ch, compare) = compose_compare(&ebytes, &abytes, &diff, ew, eh);
    let compare_path = dir.join(format!("{name}-compare.png"));
    write_rgb_png(&compare_path, cw, ch, &compare);
    let total = (ew * eh) as usize;
    let pct = (changed as f64) * 100.0 / total as f64;
    println!("\n=== {name}: {changed}/{total} pixels differ ({pct:.2}%) ===");
    println!("left = Pico-8 (green bar), middle = Nano-9 (orange bar), right = magenta mismatches");
    println!("Pico-8:  {}", expected.display());
    println!("Nano-9:  {}", actual.display());
    println!("diff:    {}", diff_path.display());
    println!("compare: {}", compare_path.display());
    let _ = std::io::stdout().flush();
    show_image(&format!("{name}: Pico-8 | Nano-9 | diff"), &compare_path);
    Ok(CartResult::Differ {
        changed,
        total,
        compare: compare_path,
    })
}

static GOLDEN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn golden_screenshots() {
    let _guard = GOLDEN_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let names = cart_names();
    assert!(!names.is_empty(), "no tests/golden/*.p8 carts found");

    let mut rows: Vec<(String, String)> = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    for name in &names {
        match check_cart(name) {
            Ok(CartResult::Match) => rows.push((name.clone(), "match".into())),
            Ok(CartResult::Differ {
                changed,
                total,
                compare,
            }) => {
                let pct = (changed as f64) * 100.0 / total as f64;
                rows.push((
                    name.clone(),
                    format!(
                        "{changed}/{total} differ ({pct:.2}%)  {}",
                        compare.display()
                    ),
                ));
                failed.push(name.clone());
            }
            Err(err) => {
                rows.push((name.clone(), format!("error: {err}")));
                failed.push(name.clone());
            }
        }
    }

    let width = rows.iter().map(|(n, _)| n.len()).max().unwrap_or(8);
    println!("\n=== golden summary ===");
    for (name, status) in &rows {
        println!("  {name:<width$}  {status}");
    }
    let _ = std::io::stdout().flush();

    if !failed.is_empty() {
        panic!(
            "{} / {} carts differ: {}",
            failed.len(),
            names.len(),
            failed.join(", ")
        );
    }
}
