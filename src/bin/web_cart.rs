//! Browser cart: `_init` / `_update` / `_draw` via luars on wasm32-unknown-unknown.
//!
//! Build and serve (from the repo root):
//! ```sh
//! trunk serve --config web/Trunk.toml
//! ```
//!
//! Requires a 16MB wasm stack (`-C link-arg=-zstack-size=16777216` in
//! `.cargo/config.toml`) and `AssetMetaCheck::Never` (set for wasm in
//! `Nano9Plugins`).

use bevy::prelude::*;
use nano9::prelude::*;

fn main() {
    App::new()
        .add_plugins(Nano9Plugins::default())
        .add_systems(Startup, load_and_insert_pico8("web-cart.lua"))
        .add_systems(PreUpdate, run_pico8_when_loaded)
        .run();
}
