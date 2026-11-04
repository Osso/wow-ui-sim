//! Port of the master-era global overrides — see
//! `src/lua_api/globals/compat_overrides.rs`.
//!
//! Covers: `print` → `SimState.console_output`, `A_Print`, `next`-on-frame
//! short-circuit, `ipairs`-on-frame children iterator. `getmetatable` /
//! `setmetatable` are intentionally not intercepted; the
//! `setmetatable_mixin_still_works` test pins that decision.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn compat_override_registration_is_idempotent() {
    let env = env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        wow_ui_sim::lua_api::globals::compat_overrides::register_all(&mut lua)
            .expect("second compat registration should succeed");
    }
    let (sum, child_count): (i64, i64) = env
        .eval(
            r#"
            local total = 0
            for _, v in pairs({ a = 1, b = 2, c = 3 }) do total = total + v end
            local parent = CreateFrame("Frame", nil, UIParent)
            CreateFrame("Frame", nil, parent)
            CreateFrame("Frame", nil, parent)
            local n = 0
            for _, child in ipairs(parent) do n = n + 1 end
            return total, n
            "#,
        )
        .unwrap();
    assert_eq!(sum, 6);
    assert_eq!(child_count, 2);
}

#[test]
fn print_writes_to_console_output() {
    let env = env();
    env.exec(r#"print("hello", 42, true)"#).unwrap();
    let output = env.state().borrow().console_output.clone();
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], "hello\t42\ttrue");
}

#[test]
fn print_handles_nil_and_tables() {
    let env = env();
    env.exec(r#"print(nil, {}, function() end)"#).unwrap();
    let output = env.state().borrow().console_output.clone();
    assert_eq!(output[0], "nil\ttable\tfunction");
}

#[test]
fn print_formats_integers_without_decimal() {
    let env = env();
    env.exec(r#"print(7, -3, 0)"#).unwrap();
    let output = env.state().borrow().console_output.clone();
    assert_eq!(output[0], "7\t-3\t0");
}

#[test]
fn a_print_routes_to_sim_print() {
    let env = env();
    env.exec(r#"A_Print("from A_Print")"#).unwrap();
    let output = env.state().borrow().console_output.clone();
    assert_eq!(output.last().map(String::as_str), Some("from A_Print"));
}

#[test]
fn a_print_is_resilient_when_print_overridden() {
    // A_Print reads the captured print from the registry — even if addon
    // code overwrites _G.print, A_Print should still work.
    let env = env();
    env.exec(
        r#"
        _G.print = function() end
        A_Print("still captured")
        "#,
    )
    .unwrap();
    let output = env.state().borrow().console_output.clone();
    assert_eq!(output.last().map(String::as_str), Some("still captured"));
}

#[test]
fn addframetext_routes_to_canonical_error_sink() {
    let env = env();
    env.exec(r#"addframetext("frame text boom")"#).unwrap();
    let state = env.state().borrow();
    assert_eq!(
        state.console_output.last().map(String::as_str),
        Some("Lua error: frame text boom")
    );
    assert_eq!(
        state.lua_error_counts.get("frame text boom"),
        Some(&1),
        "addframetext should update normalized error counts"
    );
    assert!(
        state
            .lua_errors
            .iter()
            .any(|message| message.contains("frame text boom")),
        "addframetext should record raw errors"
    );
}

#[test]
fn addframetext_formats_non_string_and_ignores_nil() {
    let env = env();
    env.exec(
        r#"
        addframetext(42)
        addframetext(nil)
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(
        state.console_output.last().map(String::as_str),
        Some("Lua error: 42")
    );
    assert_eq!(
        state.lua_error_counts.get("42"),
        Some(&1),
        "non-string addframetext values should be normalized and counted"
    );
}

#[test]
fn next_on_frame_exposes_identity_slot() {
    let env = env();
    let (key, value_type, has_second_key): (i64, String, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            local key, value = next(frame)
            return key, type(value), next(frame, key) ~= nil
            "#,
        )
        .unwrap();
    assert_eq!(key, 0, "frames expose their identity through slot zero");
    assert_eq!(value_type, "userdata");
    assert!(!has_second_key, "fresh frames expose no second raw key");
}

#[test]
fn next_on_plain_table_still_works() {
    let env = env();
    let sum: i64 = env
        .eval(
            r#"
            local total = 0
            for k, v in pairs({ a = 1, b = 2, c = 3 }) do total = total + v end
            return total
            "#,
        )
        .unwrap();
    assert_eq!(sum, 6);
}

#[test]
fn ipairs_on_frame_yields_children_in_order() {
    let env = env();
    let (c1_name, c2_name, c3_name): (String, String, String) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "IpairsParent", UIParent)
            local c1 = CreateFrame("Frame", "Child1", parent)
            local c2 = CreateFrame("Frame", "Child2", parent)
            local c3 = CreateFrame("Frame", "Child3", parent)
            local names = {}
            for i, child in ipairs(parent) do
                names[i] = child:GetName()
            end
            return names[1] or "", names[2] or "", names[3] or ""
            "#,
        )
        .unwrap();
    assert_eq!(c1_name, "Child1");
    assert_eq!(c2_name, "Child2");
    assert_eq!(c3_name, "Child3");
}

#[test]
fn ipairs_on_childless_frame_yields_zero_iterations() {
    let env = env();
    let n: i64 = env
        .eval(
            r#"
            local f = CreateFrame("Frame", nil, UIParent)
            local n = 0
            for i, child in ipairs(f) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(n, 0);
}

#[test]
fn ipairs_on_plain_array_still_works() {
    let env = env();
    let total: i64 = env
        .eval(
            r#"
            local total = 0
            for i, v in ipairs({ 10, 20, 30 }) do total = total + v end
            return total
            "#,
        )
        .unwrap();
    assert_eq!(total, 60);
}

#[test]
fn ipairs_on_non_table_reports_type_and_callsite() {
    let env = env();
    let (ok, err): (bool, String) = env
        .eval(
            r#"
            local ok, err = pcall(function()
                for _ in ipairs(1) do end
            end)
            return ok, tostring(err)
            "#,
        )
        .unwrap();
    assert!(!ok);
    assert!(
        err.contains("bad argument #1 to 'ipairs' (table expected, got number)"),
        "ipairs should keep Lua's type error shape: {err}"
    );
    assert!(
        err.contains("(string):"),
        "ipairs should report the Lua callsite: {err}"
    );
}

#[test]
fn restored_runtime_patches_match_master_shape() {
    let env = env();
    let (has_alternate_form, in_alternate_form, top, minimap, power, objective): (
        bool,
        bool,
        i64,
        i64,
        i64,
        i64,
    ) = env
        .eval(
            r#"
            UpdateUIParentPosition()
            local has_alt, in_alt = C_PlayerInfo.GetAlternateFormInfo()
            return has_alt, in_alt,
                C_UIWidgetManager.GetTopCenterWidgetSetID(),
                C_UIWidgetManager.GetBelowMinimapWidgetSetID(),
                C_UIWidgetManager.GetPowerBarWidgetSetID(),
                C_UIWidgetManager.GetObjectiveTrackerWidgetSetID()
            "#,
        )
        .unwrap();

    assert!(!has_alternate_form);
    assert!(!in_alternate_form);
    // WM14a:27 records the four native HUD getter literals.
    assert_eq!((top, minimap, power, objective), (1, 2, 283, 240));
}

#[test]
fn setmetatable_mixin_still_works() {
    // setmetatable/getmetatable aren't intercepted by this module because
    // rilua's native userdata metatable handling already supports the
    // mixin pattern. Pin that by using it here.
    let env = env();
    let greeting: String = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            setmetatable(frame, { __index = { SayHi = function() return "hi!" end } })
            return frame:SayHi()
            "#,
        )
        .unwrap();
    assert_eq!(greeting, "hi!");
}

#[test]
fn setmetatable_function_index_intercepts_method_lookup() {
    // Blizzard's menu compositor wraps frames with a *function* __index so
    // it can intercept AttachFontString / AttachTexture. The mlua build
    // needed an extra PATCH_INDEX_LUA branch to honour this; rilua's native
    // metatable dispatch handles it directly. Pin that behaviour here.
    let env = env();
    let greeting: String = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            setmetatable(frame, {
                __index = function(self, key)
                    if key == "SayHi" then
                        return function() return "function-index-hi" end
                    end
                end,
            })
            return frame:SayHi()
            "#,
        )
        .unwrap();
    assert_eq!(greeting, "function-index-hi");
}
