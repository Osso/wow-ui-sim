//! Actual Retail-selected compatibility consumers of all three Find APIs.
//! Forwarding is PRESERVATION, not proof that the pre-fix producer was correct.
//! Native output RED lives separately in spellbook_r88m4_find.rs.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common;
use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::loader::{
    discover_blizzard_addon_closure_for_screen, discover_blizzard_addons_for_screen, find_toc_file,
    load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn original_find_consumers() -> common::LockedEnv {
    common::lock_env(|| {
        let ui = blizzard_ui_dir();
        let toc_path = selected_find_owner(&ui);
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(ScreenKind::Game);
        env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
        env.exec("assert(GetCVarBool('loadDeprecationFallbacks'), 'actual default enables original owner')").unwrap();
        load_original_find_owner(&env, &ui, &toc_path);
        env.apply_post_load_workarounds();
        assert_original_find_ownership(&env);
        env
    })
}

fn selected_find_owner(ui: &std::path::Path) -> std::path::PathBuf {
    let toc_path = find_toc_file(&ui.join("Blizzard_DeprecatedSpellBook")).unwrap();
    assert!(toc_path.ends_with("Blizzard_DeprecatedSpellBook.toc"));
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_load_on_demand());
    assert!(!toc.is_game_type_restricted());
    assert!(toc.files.iter().any(|file| {
        file.file_name()
            .is_some_and(|name| name == "Deprecated_SpellBook.lua")
    }));
    let discovered = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        discovered
            .iter()
            .any(|(name, path)| { name == "Blizzard_DeprecatedSpellBook" && path == &toc_path }),
        "owner must be selected by real Game discovery"
    );
    toc_path
}

fn load_original_find_owner(env: &WowLuaEnv, ui: &std::path::Path, toc_path: &std::path::Path) {
    let closure = discover_blizzard_addon_closure_for_screen(
        &ui,
        ScreenKind::Game,
        &["Blizzard_DeprecatedSpellBook"],
    );
    assert!(
        closure
            .iter()
            .any(|(name, path)| { name == "Blizzard_DeprecatedSpellBook" && path == toc_path })
    );
    for (name, path) in closure {
        load_addon(&env.loader_env(), &path)
            .unwrap_or_else(|error| panic!("[load {name}] {error}"));
    }
}

fn assert_original_find_ownership(env: &WowLuaEnv) {
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_DeprecatedSpellBook'))
        R88OriginalFindNames = {'FindSpellOverrideByID', 'FindBaseSpellByID', 'FindFlyoutSlotBySpellID'}
        for _, name in ipairs(R88OriginalFindNames) do
            local original = _G[name]
            assert(type(original) == 'function' and original ~= C_SpellBook[name],
                name .. ' must be an original Lua consumer, not a native alias')
            local source = debug.getinfo(original, 'S').source
            assert(source:find('Blizzard_DeprecatedSpellBook/Deprecated_SpellBook.lua', 1, true),
                name .. ' original ownership missing: ' .. source)
        end
    "#).unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "original owner must load cleanly"
    );
}

#[test]
fn selected_original_find_wrappers_preserve_full_native_return_tuple() {
    test_timeout! {
        let env = original_find_consumers();
        env.exec(r#"
            local function capture(...)
                return {n = select('#', ...), ...}
            end
            for _, name in ipairs(R88OriginalFindNames) do
                for _, input in ipairs({6603, '6603.875', -1, '-1.875', 2147483647, 0}) do
                    local expected = capture(C_SpellBook[name](input))
                    local actual = capture(_G[name](input))
                    assert(actual.n == expected.n, name .. ' original lost return arity')
                    for i = 1, expected.n do
                        assert(type(actual[i]) == type(expected[i]) and actual[i] == expected[i],
                            name .. ' original changed result slot ' .. i)
                    end
                end
            end
        "#).unwrap();
    }
}

#[test]
fn selected_original_find_wrappers_preserve_required_and_range_errors() {
    test_timeout! {
        let env = original_find_consumers();
        env.exec(r#"
            for _, name in ipairs(R88OriginalFindNames) do
                local function rejects(...)
                    local ok, message = pcall(_G[name], ...)
                    assert(not ok and type(message) == 'string', name .. ' must propagate native error')
                    assert(message:find('C_SpellBook.' .. name .. '(spellID)', 1, true),
                        name .. ' missing native usage context')
                end
                rejects()
                rejects(nil)
                rejects('not a spell number')
                rejects(2147483648)
            end
        "#).unwrap();
    }
}
