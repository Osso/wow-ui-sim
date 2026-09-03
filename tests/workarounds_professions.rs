//! Integration tests for the professions panel open flow.
//!
//! Loads every Blizzard addon like the real client, then verifies that
//! casting a profession opener spell (Blacksmithing) fires the chain
//! UIParent expects and that `ProfessionsFrame` actually becomes visible.

use std::path::PathBuf;
use std::time::Duration;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::{fire_one_on_update_tick, fire_startup_events, process_pending_timers};

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn full_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1600.0, 1200.0);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    load_all_blizzard_addons(&env);
    load_professions_bootstrap_publishers(&env);
    settle_env(&env);

    env
}

fn load_all_blizzard_addons(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for (name, toc_path) in &discover_blizzard_addons(&ui) {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            panic!("[load {name}] FAILED: {err}");
        }
    }
}

fn load_professions_bootstrap_publishers(env: &WowLuaEnv) {
    let ui = blizzard_ui_dir();
    for (addon_name, bootstrap_file) in [
        ("Blizzard_Professions", "Blizzard_Professions_Bootstrap.lua"),
        (
            "Blizzard_ProfessionsBook",
            "Blizzard_ProfessionsBook_Bootstrap.lua",
        ),
    ] {
        crate::common::load_blizzard_addon_bootstrap(env, &ui, addon_name, bootstrap_file);
    }
}

fn settle_env(env: &WowLuaEnv) {
    env.apply_post_load_workarounds();
    fire_startup_events(env);
    env.apply_post_event_workarounds();
    env.state().borrow_mut().widgets.rebuild_anchor_index();
    process_pending_timers(env);
    fire_one_on_update_tick(env);
    let _ = wow_ui_sim::lua_api::globals::global_frames::hide_runtime_hidden_frames(env.lua());

    std::thread::sleep(Duration::from_secs(2));
    for _ in 0..3 {
        env.state().borrow_mut().ensure_layout_rects();
        fire_one_on_update_tick(env);
        process_pending_timers(env);
    }
}

#[test]
fn casting_blacksmithing_opens_professions_frame() {
    let env = full_game_env();
    env.state().borrow_mut().lua_errors.clear();

    let result: String = env
        .eval(
            r#"
            CastSpellByID(2018)
            if not ProfessionsFrame or not ProfessionsFrame:IsShown() then
                return "frame_not_shown"
            end
            if not ProfessionsFrame.CraftingPage or not ProfessionsFrame.CraftingPage.LinkButton then
                return "crafting_page_link_button_missing"
            end
            if ProfessionsFrame:GetWidth() < 900 then
                return "width=" .. tostring(ProfessionsFrame:GetWidth())
            end
            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "ProfessionsFrame should be visible and expanded after casting Blacksmithing (spell 2018): {result}"
    );
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "Casting Blacksmithing should not report Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
}

#[test]
fn clicking_blacksmithing_button_in_professions_book_opens_panel() {
    let env = full_game_env();
    env.state().borrow_mut().lua_errors.clear();

    let shown: bool = env
        .eval(
            r#"
            ToggleProfessionsBook()

            local button
            for _, candidate in ipairs({
                PrimaryProfession1SpellButtonTop,
                PrimaryProfession1SpellButtonBottom,
                PrimaryProfession2SpellButtonTop,
                PrimaryProfession2SpellButtonBottom,
            }) do
                if candidate then
                    local slot = ProfessionsBook_GetSpellBookItemSlot(candidate)
                    if slot then
                        local name = C_SpellBook.GetSpellBookItemName(slot, 0)
                        if name == "Blacksmithing" then
                            button = candidate
                            break
                        end
                    end
                end
            end

            assert(button, "no ProfessionsBook button is labelled 'Blacksmithing'")
            button:Click("LeftButton")

            return ProfessionsFrame ~= nil and ProfessionsFrame:IsShown() or false
            "#,
        )
        .unwrap();

    assert!(
        shown,
        "Clicking the Blacksmithing spell button should open ProfessionsFrame"
    );
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "Clicking Blacksmithing should not report Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
}
