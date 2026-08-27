//! Spike: luars 0.26.2 with `unsafe-send`, host callbacks, and Nano-9 `builtin.lua`.

use luars::{Lua, LuaApi, SafeOption, Stdlib};

fn new_lua() -> Lua {
    let mut lua = Lua::new(SafeOption::default());
    #[cfg(target_arch = "wasm32")]
    lua.open_stdlibs(&[
        Stdlib::Basic,
        Stdlib::Math,
        Stdlib::String,
        Stdlib::Table,
        Stdlib::Utf8,
        Stdlib::Coroutine,
        Stdlib::Package,
        Stdlib::Debug,
    ])
    .expect("open wasm stdlibs");
    #[cfg(not(target_arch = "wasm32"))]
    lua.open_stdlib(Stdlib::All).expect("open stdlibs");
    lua
}

#[test]
fn lua_is_send_with_unsafe_send() {
    fn assert_send<T: Send>() {}
    assert_send::<Lua>();
}

#[test]
fn host_callback_and_eval() {
    let mut lua = new_lua();
    lua.register_function("host_add", |a: i64, b: i64| a + b)
        .expect("register host_add");
    let sum: i64 = lua
        .load("return host_add(40, 2)")
        .eval()
        .expect("call host_add");
    assert_eq!(sum, 42);
}

#[test]
fn exec_nano9_builtin_lua() {
    let mut lua = new_lua();
    lua.register_function("warn", |s: String| {
        let _ = s;
    })
    .ok();
    lua.load(include_str!("../../../src/builtin.lua"))
        .set_name("builtin.lua")
        .exec()
        .unwrap_or_else(|e| panic!("builtin.lua failed: {:?}", lua.get_error_message(e)));
    let band: i64 = lua
        .load("return band(12, 10)")
        .eval()
        .expect("band from builtin.lua");
    assert_eq!(band, 8);
    let atan: f64 = lua
        .load("return atan2(1, 0)")
        .eval()
        .expect("atan2 from builtin.lua");
    assert!(atan.is_finite(), "atan2 returned {atan}");
}
