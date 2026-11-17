//! Each addon TOC is read once per UI session (QuestieDB's TOC is 57 MB).
//!
//! The kernel's per-thread read counter (`rchar` in `/proc/thread-self/io`)
//! measures the bytes this test thread reads, so the test needs no hook in
//! the loader: a second read of a large TOC shows up as its size again.

use super::*;
use std::fmt::Write as _;

const METADATA_KEYS: usize = 150_000;

fn thread_bytes_read() -> u64 {
    let io = std::fs::read_to_string("/proc/thread-self/io").expect("read thread io counters");
    io.lines()
        .find_map(|line| line.strip_prefix("rchar: "))
        .and_then(|value| value.trim().parse().ok())
        .expect("rchar counter")
}

/// Writes an addon whose TOC carries `METADATA_KEYS` metadata lines; returns the TOC size.
fn write_large_addon(root: &Path, name: &str, header: &str, lua: &str) -> u64 {
    write_large_addon_with_keys(root, name, header, lua, METADATA_KEYS)
}

fn write_large_addon_with_keys(
    root: &Path,
    name: &str,
    header: &str,
    lua: &str,
    keys: usize,
) -> u64 {
    let mut toc = format!(
        "## Interface: {}\n## Title: {name} title\n{header}",
        wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
    );
    for key in 0..keys {
        writeln!(toc, "## X-{name}-{key}: value-{key}").unwrap();
    }
    toc.push_str("main.lua\n");
    let addon_dir = root.join(name);
    std::fs::create_dir_all(&addon_dir).expect("create addon dir");
    std::fs::write(addon_dir.join(format!("{name}.toc")), &toc).expect("write toc");
    std::fs::write(addon_dir.join("main.lua"), lua).expect("write lua");
    toc.len() as u64
}

/// The first Lua load in a process reads the shared bytecode-cache pack (hundreds of
/// MB). Load a small addon in a separate environment so the measurement excludes it.
fn warm_process_caches() {
    let temp = tempfile::tempdir().unwrap();
    write_large_addon_with_keys(temp.path(), "Warmup", "", "WarmupRuns = 1", 1);
    let env = WowLuaEnv::new().unwrap();
    let roots = [temp.path().to_path_buf()];
    load_third_party_addons(&roots, false, false, &env, &mut None, ScreenKind::Game);
}

const CHECK_STARTUP_LUA: &str = r#"
assert(BigStartupRuns == 1 and BigLodRuns == nil)
assert(C_AddOns.IsAddOnLoaded('BigStartup'))
assert(not C_AddOns.IsAddOnLoaded('BigLod'))
assert(C_AddOns.IsAddOnLoadOnDemand('BigLod'))
assert(C_AddOns.GetAddOnMetadata('BigStartup', 'x-bigstartup-149999') == 'value-149999')
assert(C_AddOns.GetAddOnMetadata('BigLod', 'Title') == 'BigLod title')
"#;

const LOAD_LOD_LUA: &str = r#"
assert(C_AddOns.LoadAddOn('BigLod'))
assert(BigLodRuns == 1 and C_AddOns.IsAddOnLoaded('BigLod'))
assert(C_AddOns.GetAddOnMetadata('BigLod', 'X-BigLod-7') == 'value-7')
"#;

#[test]
fn startup_and_load_on_demand_read_each_large_toc_once() {
    warm_process_caches();
    let temp = tempfile::tempdir().unwrap();
    let startup_size = write_large_addon(
        temp.path(),
        "BigStartup",
        "",
        "BigStartupRuns = (BigStartupRuns or 0) + 1",
    );
    let lod_size = write_large_addon(
        temp.path(),
        "BigLod",
        "## LoadOnDemand: 1\n",
        "BigLodRuns = (BigLodRuns or 0) + 1",
    );
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().addon_base_paths = vec![temp.path().to_path_buf()];
    let roots = [temp.path().to_path_buf()];

    let before_startup = thread_bytes_read();
    load_third_party_addons(&roots, false, false, &env, &mut None, ScreenKind::Game);
    let startup_read = thread_bytes_read() - before_startup;
    env.exec(CHECK_STARTUP_LUA).unwrap();
    let before_load = thread_bytes_read();
    env.exec(LOAD_LOD_LUA).unwrap();
    let load_read = thread_bytes_read() - before_load;

    let tocs = startup_size + lod_size;
    eprintln!("TOCs {tocs} bytes; startup read {startup_read}, LoadAddOn read {load_read}");
    assert!(
        startup_read >= tocs,
        "the read counter missed the TOC reads: {startup_read} < {tocs} bytes"
    );
    assert!(
        startup_read < tocs + tocs / 2,
        "startup read {startup_read} bytes for {tocs} bytes of TOCs: a TOC was read again"
    );
    assert!(
        load_read < lod_size / 2,
        "LoadAddOn read {load_read} bytes after startup read the {lod_size}-byte TOC"
    );
}
