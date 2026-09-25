use std::fs;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn frozen_addon_namespace_does_not_freeze_callback_state_or_later_addons() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("FreezeNamespaceProbe");
    let second = directory.path().join("AfterFreezeProbe");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    fs::write(
        first.join("FreezeNamespaceProbe.toc"),
        "## Title: Freeze namespace probe\nFreezeNamespaceProbe.lua\n",
    )
    .unwrap();
    fs::write(
        first.join("FreezeNamespaceProbe.lua"),
        r#"
        local _, addon = ...
        local pending = { AfterFreezeProbe = "waiting" }
        local child = { label = "child-alive" }
        addon.child = child
        addon.onLoaded = function(name)
            if name == "AfterFreezeProbe" then
                pending[name] = nil
                return child.label
            end
        end
        FreezeNamespaceProbe = addon
        FreezeLoadedEvents = {}
        local frame = CreateFrame("Frame")
        frame:RegisterEvent("ADDON_LOADED")
        frame:SetScript("OnEvent", function(_, _, name)
            FreezeLoadedEvents[#FreezeLoadedEvents + 1] = name
            if name == "AfterFreezeProbe" then
                FreezeCallbackResult = addon.onLoaded(name)
                FreezePendingCleared = pending[name] == nil
            end
        end)
        table.freeze(addon)
        "#,
    )
    .unwrap();
    fs::write(
        second.join("AfterFreezeProbe.toc"),
        "## Title: After freeze probe\nAfterFreezeProbe.lua\n",
    )
    .unwrap();
    fs::write(
        second.join("AfterFreezeProbe.lua"),
        r#"
        AfterFreezeGlobal = "loaded-after-freeze"
        AfterFreezeMatch = string.match("callback-alive", "callback")
        "#,
    )
    .unwrap();

    let env = env();
    let loaded = load_addon(&env.loader_env(), &first.join("FreezeNamespaceProbe.toc"))
        .expect("first addon chunk should load");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("FreezeNamespaceProbe")])
        .expect("first addon callback should dispatch");
    let loaded = load_addon(&env.loader_env(), &second.join("AfterFreezeProbe.toc"))
        .expect("later addon chunk should load after freeze");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    env.fire_event_with_args("ADDON_LOADED", &[env.lua_string("AfterFreezeProbe")])
        .expect("later addon callback should dispatch");
    env.exec(
        r#"
        assert(table.concat(FreezeLoadedEvents, ",") == "FreezeNamespaceProbe,AfterFreezeProbe")
        assert(FreezePendingCleared == true)
        assert(FreezeCallbackResult == "child-alive")
        assert(AfterFreezeGlobal == "loaded-after-freeze")
        assert(AfterFreezeMatch == "callback")
        collectgarbage("collect")
        assert(FreezeNamespaceProbe.onLoaded("AfterFreezeProbe") == "child-alive")
        assert(FreezeNamespaceProbe.child.label == "child-alive")
        assert(string.match("still-alive", "alive") == "alive")
        "#,
    )
    .expect("callback state, closure strings and children survive full GC");
    assert!(env.state().borrow().lua_errors.is_empty());
}

#[test]
fn table_count_counts_all_non_nil_entries() {
    let env = env();
    let counts: (i32, i32, i32) = env
        .eval(
            r#"
            local labels = {
                "first",
                "second",
                [4] = "fourth",
                rank = "leader",
                skipped = nil,
            }
            return table.count(labels)
            "#,
        )
        .unwrap();
    assert_eq!(
        counts,
        (4, 3, 4),
        "table.count should return total nodes, array nodes, and max array index"
    );
}

#[test]
fn table_count_returns_nothing_for_non_tables() {
    let env = env();
    let returned_count: i32 = env
        .eval("return select('#', table.count('not-table'))")
        .unwrap();
    assert_eq!(returned_count, 0, "non-table inputs should return nothing");
}

#[test]
fn table_count_errors_for_nil() {
    let env = env();
    let ok: bool = env.eval("return pcall(table.count, nil)").unwrap();
    assert!(!ok, "nil input should raise instead of returning nothing");
}

#[test]
fn table_count_errors_for_missing_argument() {
    let env = env();
    let ok: bool = env.eval("return pcall(table.count)").unwrap();
    assert!(!ok, "missing input should raise like nil");
}

#[test]
fn table_util_find_indexed_mismatch_returns_nil_for_equal_arrays() {
    let env = env();
    let mismatch_index: Option<i32> = env
        .eval(
            r#"
            local t1 = { 1, 2, 3, 4 }
            local t2 = { 1, 2, 3, 4 }
            return C_TableUtil.FindIndexedMismatch(t1, t2)
            "#,
        )
        .unwrap();
    assert_eq!(mismatch_index, None, "equal arrays should not mismatch");
}

#[test]
fn table_util_find_indexed_mismatch_returns_first_mismatch_index() {
    let env = env();
    let mismatch_index: Option<i32> = env
        .eval(
            r#"
            local t1 = { "a", "b", "c", "d" }
            local t2 = { "a", "x", "c", "d" }
            return C_TableUtil.FindIndexedMismatch(t1, t2)
            "#,
        )
        .unwrap();
    assert_eq!(
        mismatch_index,
        Some(2),
        "first differing element should be reported"
    );
}

#[test]
fn table_util_find_indexed_mismatch_detects_length_difference() {
    let env = env();
    let mismatch_index: Option<i32> = env
        .eval(
            r#"
            local t1 = { 10, 20, 30 }
            local t2 = { 10, 20, 30, 40 }
            return C_TableUtil.FindIndexedMismatch(t1, t2)
            "#,
        )
        .unwrap();
    assert_eq!(
        mismatch_index,
        Some(4),
        "extra entries should be reported as mismatch at first missing index"
    );
}

#[test]
fn table_util_find_indexed_mismatch_supports_comparator_function() {
    let env = env();
    let mismatch_index: Option<i32> = env
        .eval(
            r#"
            local t1 = { "Alpha", "Beta", "Gamma" }
            local t2 = { "alpha", "BETA", "delta" }
            local function comparator(v1, v2, index)
                return string.lower(v1) == string.lower(v2)
            end
            return C_TableUtil.FindIndexedMismatch(t1, t2, comparator)
            "#,
        )
        .unwrap();
    assert_eq!(
        mismatch_index,
        Some(3),
        "comparator should control equality check for each indexed element"
    );
}

#[test]
fn table_util_find_indexed_mismatch_returns_nil_for_non_tables() {
    let env = env();
    let mismatch_index: Option<i32> = env
        .eval("return C_TableUtil.FindIndexedMismatch('not-table', { 1, 2, 3 })")
        .unwrap();
    assert_eq!(mismatch_index, None, "non-table inputs should return nil");
}
