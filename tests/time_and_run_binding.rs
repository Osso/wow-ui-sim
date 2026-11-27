//! Rust `time` (OsDocumentation.lua) and `RunBinding` (retail-12-1-5-70688.csv
//! `_G,RunBinding`; Forever consumer StackSplitFrame.lua:259).
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn time_converts_date_tables_like_os_time() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(
        r#"
        local first = time({ year = 2024, month = 1, day = 2, hour = 3, min = 4, sec = 5 })
        local second = time({ year = "2024", month = "1", day = "3", hour = "3", min = "4", sec = "5" })
        assert(second - first == 86400, "one day apart: " .. tostring(second - first))
        assert(first == os.time({ year = 2024, month = 1, day = 2, hour = 3, min = 4, sec = 5 }))
        local now = time()
        assert(math.abs(now - os.time()) <= 1, "time() is the current Unix time")
        "#,
    )
    .expect("time follows its date table");
}

#[test]
fn run_binding_runs_the_declared_body_with_the_requested_key_state() {
    let directory = tempfile::tempdir().expect("temporary addon directory");
    std::fs::write(
        directory.path().join("Bindings.xml"),
        r#"<Bindings>
            <Binding name="RUN_A">RunTrace = (RunTrace or '') .. 'A' .. keystate .. ','</Binding>
            <Binding name="RUN_B">RunTrace = (RunTrace or '') .. 'B' .. keystate .. ','</Binding>
        </Bindings>"#,
    )
    .expect("write binding file");
    let toc = directory.path().join("RunBindingTest.toc");
    std::fs::write(&toc, "Bindings.xml\n").expect("write addon TOC");
    let env = WowLuaEnv::new().expect("Lua environment");
    load_addon(&env.loader_env(), &toc).expect("load addon");
    env.exec("RunBinding('RUN_A'); RunBinding('RUN_B', 'up'); RunBinding('RUN_A', 'down')")
        .expect("run bindings");
    let trace: String = env.eval("return RunTrace").expect("binding trace");
    assert_eq!(trace, "Adown,Bup,Adown,");
}
