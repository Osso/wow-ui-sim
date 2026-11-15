use crate::common;

use std::{cell::RefCell, rc::Rc};
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::render::WowFontSystem;

fn blizzard_ui_dir() -> std::path::PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be synced")
}

fn setup_full_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_font_system(Rc::new(RefCell::new(WowFontSystem::new())));
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    if cfg!(feature = "client-wrath") {
        common::wrath_watch_fixture::seed_watch_quests(&env);
    }
    common::load_profile_framexml(&env);
    for (name, toc_path) in &discover_blizzard_addons(&ui) {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {err}");
        }
    }

    env.apply_post_load_workarounds();
    if cfg!(feature = "client-wrath") {
        env.exec(include_str!("objective_tracker/wrath_watch.lua"))
            .unwrap();
    }
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

#[test]
fn quest_objective_tracker_populates_titles_via_normal_async_load_path() {
    let env = setup_full_ui();

    let titles: String = env
        .eval(if cfg!(feature = "client-wrath") {
            include_str!("objective_tracker/wrath_titles.lua")
        } else {
            r#"
            local titles = {}
            for _, blocks in pairs(QuestObjectiveTracker.usedBlocks or {}) do
                for _, block in pairs(blocks) do
                    local text = block.HeaderText and block.HeaderText:GetText()
                    if text and text ~= "" then
                        table.insert(titles, text)
                    end
                end
            end
            table.sort(titles)
            return table.concat(titles, "|")
            "#
        })
        .unwrap();

    // VERIFIED: Camelot prefixes each quest title with its difficulty level.
    // Retail seeds two ordinary quests; the Wrath watch fixture supplies Supply Run too.
    let expected = if cfg!(feature = "client-wrath") {
        "Defending the Gates|Supply Run|The Lost Expedition"
    } else if cfg!(feature = "client-wowforever") {
        "[80] Defending the Gates|[80] The Lost Expedition"
    } else {
        "Defending the Gates|The Lost Expedition"
    };
    assert_eq!(titles, expected);
}

#[test]
fn quest_objective_tracker_measures_objective_line_text_height() {
    let env = setup_full_ui();

    let collapsed_lines: String = env
        .eval(if cfg!(feature = "client-wrath") {
            include_str!("objective_tracker/wrath_line_heights.lua")
        } else {
            r#"
            local collapsed = {}
            for _, blocks in pairs(QuestObjectiveTracker.usedBlocks or {}) do
                for _, block in pairs(blocks) do
                    for _, lineFrame in pairs(block.usedLines or {}) do
                        local textFs = lineFrame.Text
                        local text = textFs and textFs:GetText()
                        if text and text ~= "" and textFs:GetHeight() <= 0 then
                            table.insert(collapsed, text)
                        end
                    end
                end
            end
            table.sort(collapsed)
            return table.concat(collapsed, "|")
            "#
        })
        .unwrap();

    assert_eq!(
        collapsed_lines, "",
        "objective line text should be measured, not collapsed to zero height"
    );
}
