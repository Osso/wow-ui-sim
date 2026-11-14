//! Original Retail spellbook AutoCast PRESERVATION, not an arity RED.
//! A false/false constant also passes this no-pet-state visual case; decisive
//! R88m.4 zero/two-result RED selectors live in spellbook_r88m4_items.rs.
//! No substitute item data, native API callbacks, XML frames or vendor edits.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, find_toc_file,
    load_startup_addon, register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn original_book_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc = find_toc_file(&ui.join("Blizzard_PlayerSpells")).expect("selected spellbook TOC");
    assert!(toc.ends_with("Blizzard_PlayerSpells.toc"));
    let source = std::fs::read_to_string(&toc).unwrap();
    assert!(source.contains("## LoadOnDemand: 1"));
    assert!(source.contains("SpellBook/Blizzard_SpellBookItem.xml"));
    assert!(source.contains("SpellBook/Blizzard_SpellBookFrame.xml"));
    assert!(source.contains("Blizzard_PlayerSpellsFrame.xml"));
    let item_xml = std::fs::read_to_string(
        ui.join("Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookItem.xml"),
    )
    .unwrap();
    assert!(item_xml.contains("<Script file=\"Blizzard_SpellBookItem.lua\"/>"));
    assert!(item_xml.contains("mixin=\"SpellBookItemMixin\""));
    assert!(item_xml.contains("parentKey=\"AutoCastOverlay\""));
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    for addon in discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game) {
        if !env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|a| a.folder_name == addon.name && a.loaded)
        {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn assert_original_visible_item_autocast(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(PlayerSpellsFrame:IsShown())
        local book = PlayerSpellsFrame.SpellBookFrame
        assert(book:IsShown())
        assert(debug.getinfo(SpellBookItemMixin.UpdateAutoCast).source:find(
            'Blizzard_PlayerSpells/SpellBook/Blizzard_SpellBookItem.lua', 1, true))
        local checked = 0
        for _, item in book.PagedSpellsFrame:EnumerateFrames() do
            if item:IsShown() and item.HasValidData and item:HasValidData()
                and not item.isOffSpec and item.spellBank == Enum.SpellBookSpellBank.Player
                and item.Button and item.Button.AutoCastOverlay then
                assert(item.UpdateAutoCast == SpellBookItemMixin.UpdateAutoCast)
                assert(item:GetScript('OnEvent'))
                -- Invoke the actual installed owner on its original paged item,
                -- whose Init/UpdateSpellData already obtained all book data.
                item:UpdateAutoCast()
                assert(not item.Button.AutoCastOverlay:IsShown())
                assert(not item.Button.AutoCastOverlay.Shine:IsShown())
                checked = checked + 1
            end
        end
        assert(checked > 0, 'original paged book must supply visible player spell items')
        "#,
    )
    .expect("original AutoCast owner keeps no-candidate player items unadorned");
}

#[test]
fn original_book_autocast_preserves_hidden_overlay_on_open_and_reopen() {
    let env = original_book_env();
    assert!(env.state().borrow().pet_spells.is_empty());
    assert!(
        env.state()
            .borrow()
            .pet_actions
            .iter()
            .all(|a| !a.has_action)
    );
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("original demand-load/open route");
    assert_original_visible_item_autocast(&env);
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("original close route");
    env.exec("assert(not PlayerSpellsFrame:IsShown())")
        .expect("book closes without forced visibility");
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("original reopen route");
    assert_original_visible_item_autocast(&env);
    for error in &env.state().borrow().lua_error_records {
        println!(
            "R88m.4 retained startup/consumer diagnostic: {}",
            error.message
        );
        assert!(
            !error.message.contains("Blizzard_SpellBookItem.lua")
                && !error.message.contains("Blizzard_SpellBookFrame.lua")
                && !error.message.contains("Blizzard_PlayerSpellsFrame.lua"),
            "original book owner error: {}",
            error.message
        );
    }
}

#[test]
fn original_deprecated_loc_wrapper_preserves_required_bank_rejection() {
    // This original adapter supplies only one positional argument. It cannot
    // demonstrate a successful two-position native query without modification.
    // Keep its existing rejection visible; do not patch the adapter to make a
    // happy-path LoC consumer proof appear.
    let ui = blizzard_ui_dir();
    let toc = find_toc_file(&ui.join("Blizzard_Deprecated")).unwrap();
    assert!(toc.ends_with("Blizzard_Deprecated.toc"));
    let source = std::fs::read_to_string(&toc).unwrap();
    assert!(source.contains("Mainline/Deprecated_12_0_1.lua [AllowLoadGameType mainline]"));
    assert!(!source.contains("## LoadOnDemand: 1"));
    let env = original_book_env();
    env.exec(
        r#"
        assert(GetCVarBool('loadDeprecationFallbacks'), 'default fallback publication gate')
        local wrapper = C_SpellBook.GetSpellBookItemLossOfControlCooldown
        assert(type(wrapper) == 'function')
        assert(debug.getinfo(wrapper).source:find(
            'Blizzard_Deprecated/Mainline/Deprecated_12_0_1.lua', 1, true))
        local ok, err = pcall(wrapper, 1)
        assert(not ok and type(err) == 'string')
        assert(string.find(err, 'Usage: local lossOfControlInfo = C_SpellBook.GetSpellBookItemLossOfControlCooldownInfo(spellBookItem)', 1, true), err)
        -- Passing another argument to the wrapper cannot change its one-arg
        -- forwarding body. This is adapter preservation, not a native success.
        local extraOK, extraError = pcall(wrapper, 1, Enum.SpellBookSpellBank.Player)
        assert(not extraOK and type(extraError) == 'string')
        assert(string.find(extraError, 'GetSpellBookItemLossOfControlCooldownInfo', 1, true))
        "#,
    )
    .expect("selected original one-argument fallback reaches required-bank rejection");
}
