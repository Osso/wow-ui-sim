#![cfg(all(
    feature = "optional-tests",
    any(feature = "client-retail", feature = "client-wowforever")
))]

use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use crate::common::blizzard_addon_harness::new_blizzard_addon_env;
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn opened_spellbook_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    load_startup_addons(&env, &ui);
    env.sync_string_metatable_to_global_string();
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    supply_forever_spellbook_inputs(&env);
    env.exec("PlayerSpellsUtil.ToggleSpellBookFrame()")
        .expect("load and open the original PlayerSpells book");
    env
}

fn supply_forever_spellbook_inputs(env: &WowLuaEnv) {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever {
        let mut state = env.state().borrow_mut();
        state.transmog_availability.enabled = Some(false);
        state.transmog_outfit_catalog.entries.clear();
        state.transmog_outfit_catalog.outfits_info_supplied = Some(false);
    }
}

fn load_startup_addons(env: &WowLuaEnv, ui: &std::path::Path) {
    for addon in discover_blizzard_startup_addons_for_screen(ui, ScreenKind::Game) {
        let loaded = env
            .state()
            .borrow()
            .addons
            .iter()
            .any(|loaded| loaded.folder_name == addon.name && loaded.loaded);
        if !loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None).unwrap();
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
}

#[test]
fn original_spellbook_autocast_update_uses_each_supplied_row() {
    let env = opened_spellbook_env();
    assert_supplied_autocast_visuals(&env);
    assert_autocast_mutations_update_visuals(&env);
}

#[test]
fn supplied_pet_spellbook_autocast_can_be_toggled() {
    let env = WowLuaEnv::new().expect("pet spellbook autocast API environment");
    env.exec(
        r#"
        local pet = Enum.SpellBookSpellBank.Pet
        A_Admin.SetSpellBookItemAutoCast(1, pet, true, false)
        local allowed, enabled = C_SpellBook.GetSpellBookItemAutoCast(1, pet)
        assert(allowed and not enabled)
        C_SpellBook.SetSpellBookItemAutoCastEnabled(1, pet, true)
        assert(select(2, C_SpellBook.GetSpellBookItemAutoCast(1, pet)))
        C_SpellBook.ToggleSpellBookItemAutoCast(1, pet)
        assert(not select(2, C_SpellBook.GetSpellBookItemAutoCast(1, pet)))
        A_Admin.SetSpellBookItemAutoCast(2, pet, false, false)
        C_SpellBook.ToggleSpellBookItemAutoCast(2, pet)
        local disallowed, disabled = C_SpellBook.GetSpellBookItemAutoCast(2, pet)
        assert(not disallowed and not disabled)
        "#,
    )
    .expect("explicit pet rows retain their supplied autocast state");
}

fn assert_supplied_autocast_visuals(env: &WowLuaEnv) {
    env.exec(
        r#"
        local items = {}
        for _, item in PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame:EnumerateFrames() do
            if item:IsShown() and item.HasValidData and item:HasValidData()
                and not item.isOffSpec and item.spellBank == Enum.SpellBookSpellBank.Player
                and item.Button and item.Button.AutoCastOverlay then
                items[#items + 1] = item
            end
        end
        assert(#items >= 3, 'original page must show three valid player spell rows')
        A_Admin.SetSpellBookItemAutoCast(items[1].slotIndex, items[1].spellBank, true, true)
        A_Admin.SetSpellBookItemAutoCast(items[2].slotIndex, items[2].spellBank, true, false)
        A_Admin.SetSpellBookItemAutoCast(items[3].slotIndex, items[3].spellBank, false, false)
        for index = 1, 3 do items[index]:UpdateAutoCast() end
        assert(items[1].Button.AutoCastOverlay:IsShown(), 'allowed enabled row shows overlay')
        assert(items[1].Button.AutoCastOverlay.Shine:IsShown(), 'enabled row shows shine')
        assert(items[2].Button.AutoCastOverlay:IsShown(), 'allowed disabled row shows overlay')
        assert(not items[2].Button.AutoCastOverlay.Shine:IsShown(), 'disabled row hides shine')
        assert(not items[3].Button.AutoCastOverlay:IsShown(), 'disallowed row hides overlay')
        assert(not items[3].Button.AutoCastOverlay.Shine:IsShown(), 'disallowed row hides shine')
        "#,
    )
    .expect("the original item update must render supplied autocast flags per row");
}

fn assert_autocast_mutations_update_visuals(env: &WowLuaEnv) {
    env.exec(
        r#"
        for _, item in PlayerSpellsFrame.SpellBookFrame.PagedSpellsFrame:EnumerateFrames() do
            if item:IsShown() and item.HasValidData and item:HasValidData()
                and item.spellBank == Enum.SpellBookSpellBank.Player
                and item.Button and item.Button.AutoCastOverlay then
                A_Admin.SetSpellBookItemAutoCast(item.slotIndex, item.spellBank, true, false)
                item:UpdateAutoCast()
                assert(not item.Button.AutoCastOverlay.Shine:IsShown())
                C_SpellBook.SetSpellBookItemAutoCastEnabled(item.slotIndex, item.spellBank, true)
                item:UpdateAutoCast()
                assert(item.Button.AutoCastOverlay.Shine:IsShown(), 'set-enabled turns shine on')
                C_SpellBook.ToggleSpellBookItemAutoCast(item.slotIndex, item.spellBank)
                item:UpdateAutoCast()
                assert(not item.Button.AutoCastOverlay.Shine:IsShown(), 'toggle turns shine off')
                return
            end
        end
        error('no visible spell row to test autocast mutations')
        "#,
    )
    .expect("spellbook autocast mutations update the original row visuals");
}
