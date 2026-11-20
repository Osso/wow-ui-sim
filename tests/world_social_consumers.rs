#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

fn loaded_game_addons() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.state().borrow_mut().addon_base_paths = vec![common::blizzard_ui_dir()];
    common::supply_empty_forever_spellbook_outfits(&env);
    common::load_game_startup_inventory(&env);
    env.apply_post_load_workarounds();
    env
}

#[test]
fn original_spellbook_defers_world_events_until_world_entry() {
    test_timeout! {
        let env = loaded_game_addons();
        env.exec(r#"
            C_AddOns.LoadAddOn("Blizzard_PlayerSpells")
            local book = PlayerSpellsFrame.SpellBookFrame
            assert(not book:IsEventRegistered("LEARNED_SPELL_IN_SKILL_LINE"),
                "original OnLoad must defer world-only events before entry")
            book:HookScript("OnEvent", function(self, event)
                if event == "PLAYER_ENTERING_WORLD" then
                    self.worldEntryObserved = IsPlayerInWorld()
                end
            end)
        "#).expect("load-time original spellbook consumer");
        wow_ui_sim::startup::settle_headless_startup(&env);
        env.exec(r#"
            local book = PlayerSpellsFrame.SpellBookFrame
            assert(book.worldEntryObserved == true, "world state must be true inside entry handler")
            assert(book:IsEventRegistered("LEARNED_SPELL_IN_SKILL_LINE"),
                "original entry handler must register world-only events")
        "#).expect("world-entry original spellbook consumer");
    }
}
