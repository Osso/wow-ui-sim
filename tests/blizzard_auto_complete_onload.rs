use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
use wow_ui_sim::loader::load_addon;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
use wow_ui_sim::screen::ScreenKind;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
use wow_ui_sim::startup::settle_headless_startup;

const ROOT: &str = "Blizzard_AutoComplete";
const ONLOAD_PROBE_LUA: &str = r##"
local failures = {}

local function expect(condition, message)
  if not condition then
    table.insert(failures, message)
  end
end

expect(type(AutoComplete_OnLoad) == "function",
       "AutoComplete_OnLoad must be a function")
expect(AutoCompleteBox ~= nil, "AutoCompleteBox must exist")
expect(AutoCompleteButton1 ~= nil, "AutoCompleteButton1 must exist")
expect(AutoCompleteInstructions ~= nil, "AutoCompleteInstructions must exist")

if AutoCompleteBox == nil or AutoCompleteButton1 == nil or AutoCompleteInstructions == nil then
  return table.concat(failures, "\n")
end

AutoComplete_OnLoad(AutoCompleteBox)

local expectedHeight = 5 * AutoCompleteButton1:GetHeight()
expect(AutoCompleteBox.maxHeight == expectedHeight,
       "AutoCompleteBox.maxHeight must be 5 * AutoCompleteButton1:GetHeight(), got " ..
       tostring(AutoCompleteBox.maxHeight) .. " expected " .. tostring(expectedHeight))
expect(AutoCompleteInstructions:GetText() == "|cffbbbbbb" .. PRESS_TAB .. "|r",
       "AutoCompleteInstructions text mismatch")
expect(AutoCompleteBox:IsEventRegistered("GUILD_ROSTER_UPDATE"),
       "AutoCompleteBox must register GUILD_ROSTER_UPDATE")

return table.concat(failures, "\n")
"##;

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
fn seed_secure_autocomplete_name(env: &WowLuaEnv) {
    env.exec(
        r#"
        local sentinel = UIParent:CreateFontString(nil, "OVERLAY")
        rawset(_G, "__testAutoCompleteSecureSentinel", sentinel)
        rawset(__secureenv, "AutoCompleteInstructions", sentinel)
        "#,
    )
    .expect("seed an existing secure AutoComplete name");
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
fn load_login_addons(env: &WowLuaEnv, ui: &std::path::Path) {
    for (name, toc_path) in discover_blizzard_addons_for_screen(ui, ScreenKind::Login) {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
fn load_login_screen_with_autocomplete_preloaded(seed_secure_name: bool) -> common::LockedEnv {
    common::lock_env(move || {
        let env = WowLuaEnv::new().expect("create Retail login environment");
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(ScreenKind::Login);
        if seed_secure_name {
            seed_secure_autocomplete_name(&env);
        }

        let ui = common::blizzard_ui_dir();
        let autocomplete_toc = ui.join("Blizzard_AutoComplete/Blizzard_AutoComplete.toc");
        load_addon(&env.loader_env(), &autocomplete_toc)
            .expect("ordinary AutoComplete request should load");
        env.exec(
            "rawset(_G, '__ordinaryAutoCompleteInstructions', AutoCompleteInstructions)",
        )
        .expect("save the ordinary AutoComplete FontString");
        load_login_addons(&env, &ui);
        env.apply_post_load_workarounds();
        settle_headless_startup(&env);
        env
    })
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
fn auto_complete_secure_probe(env: &WowLuaEnv) -> (bool, bool, bool, bool, bool) {
    let load_modes = {
        let state = env.state().borrow();
        state
            .addons
            .iter()
            .find(|addon| addon.folder_name == ROOT)
            .map(|addon| (addon.loaded, addon.secure_loaded))
    };
    assert_eq!(load_modes, Some((true, true)), "AutoComplete loads in both modes");
    env.eval(
        r#"
        local instructions = rawget(__secureenv, "AutoCompleteInstructions")
        local box = rawget(__secureenv, "AutoCompleteBox")
        return instructions ~= nil,
            type(box) == "table" and type(box.maxHeight) == "number" and box.maxHeight > 0,
            rawequal(rawget(_G, "__ordinaryAutoCompleteInstructions"), rawget(_G, "AutoCompleteInstructions")),
            rawequal(rawget(_G, "CatalogShopFrame"), rawget(__secureenv, "CatalogShopFrame")),
            rawget(_G, "__testAutoCompleteSecureSentinel") == nil
                or rawequal(instructions, rawget(_G, "__testAutoCompleteSecureSentinel"))
        "#,
    )
    .expect("AutoComplete secure dependency probe should run")
}

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
#[test]
fn retail_catalog_shop_secure_dependency_publishes_autocomplete_names_without_overwrite() {
    test_timeout! {
        let env = load_login_screen_with_autocomplete_preloaded(false);
        let vacant_names = auto_complete_secure_probe(&env);
        let vacant_errors = env.state().borrow().lua_errors.clone();
        drop(env);

        let env = load_login_screen_with_autocomplete_preloaded(true);
        let occupied_name = auto_complete_secure_probe(&env);
        let occupied_errors = env.state().borrow().lua_errors.clone();
        assert_eq!(vacant_names, (true, true, true, true, true));
        assert_eq!(occupied_name, (true, true, true, true, true));
        assert!(vacant_errors.iter().all(|error| !error.contains("AutoComplete.lua:133")));
        assert!(occupied_errors.iter().all(|error| !error.contains("AutoComplete.lua:133")));
    }
}

#[test]
fn blizzard_auto_complete_onload_initializes_box_state() {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        common::wrath_autocomplete::load_and_assert_native(
            r#"
AutoComplete_OnLoad(AutoCompleteBox)
assert(AutoCompleteBox.maxHeight == 5 * AutoCompleteButton1:GetHeight())
assert(AutoCompleteInstructions:GetText() == '|cffbbbbbb'..PRESS_TAB..'|r')
assert(not AutoCompleteBox:IsEventRegistered('GUILD_ROSTER_UPDATE'))
"#,
        );
        return;
    }

    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[ROOT], &[], |env, loaded| {
                assert!(
                    loaded.iter().any(|name| name == ROOT),
                    "`{ROOT}` must load before AutoComplete_OnLoad can be checked. \
                     Loaded set: {loaded:?}"
                );

                let failures: String = env
                    .eval(ONLOAD_PROBE_LUA)
                    .expect("AutoComplete OnLoad probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` OnLoad mismatches:\n{failures}"
                );
            });
        });
    });
}
