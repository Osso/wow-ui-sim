//! INFERRED Forever projection of supplied PTR-model item cooldowns.
//! Original selected Collections consumer; no vendor or API replacement.
#![cfg(all(feature = "client-wowforever", feature = "optional-tests"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use sha2::{Digest, Sha256};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen;
use wow_ui_sim::lua_api::{WowLuaEnv, state::SpellCooldownState};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const ORIGINAL_SOURCE: &str = r#"
    local function original(fn, suffix)
        local info = debug.getinfo(fn, 'S')
        assert(info.what == 'Lua')
        assert(info.source:gsub('\\', '/'):sub(-#suffix) == suffix)
    end
    original(CollectionsSpellButton_UpdateCooldown,
        'Blizzard_Collections/Shared/Blizzard_CollectionTemplates.lua')
    original(ToySpellButton_UpdateButton,
        'Blizzard_Collections/Mainline/Blizzard_ToyBox.lua')
    assert(C_AddOns.IsAddOnLoaded('Blizzard_Collections'))
"#;

const OBSERVE_COOLDOWNS: &str = r#"
    local function observe(itemID, start, duration)
        local button = CreateFrame('Button', nil, UIParent)
        button.itemID = itemID
        button.cooldown = CreateFrame('Cooldown', nil, button)
        button.cooldown:Show()
        CollectionsSpellButton_UpdateCooldown(button)
        if start then
            assert(button.cooldown:IsShown(), 'missing cooldown for item '..itemID)
            local actualStart, actualDuration = button.cooldown:GetCooldownTimes()
            assert(actualStart == start * 1000,
                'item '..itemID..' start='..tostring(actualStart)..', expected='..start*1000)
            assert(actualDuration == duration * 1000,
                'item '..itemID..' duration='..tostring(actualDuration)..', expected='..duration*1000)
        else
            assert(not button.cooldown:IsShown(), 'absent item cooldown must hide')
        end
    end
    observe(6948, 11, 31)
    observe(140192, 17, 43)
    observe(880001, nil, nil)
"#;

fn assert_selected_files(toc: &TocFile) {
    assert!(toc.is_load_on_demand());
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    for expected in [
        "Shared/Blizzard_CollectionTemplates.lua",
        "Mainline/Blizzard_ToyBox.lua",
    ] {
        assert!(
            toc.files.iter().enumerate().any(|(i, file)| {
                file.to_string_lossy().replace('\\', "/") == expected
                    && toc.file_allows_screen(i, ScreenKind::Game)
            }),
            "missing selected file {expected}"
        );
    }
}

fn assert_source_hashes(ui: &std::path::Path) {
    for (relative, digest) in [
        (
            "Blizzard_Collections/Blizzard_Collections.toc",
            "4babc1ed336654d3a5e0f2a0a58b1e38c9dc289fed6a047538bbd5e380fd5e45",
        ),
        (
            "Blizzard_Collections/Shared/Blizzard_CollectionTemplates.lua",
            "99aa2918d9a923c421ec33660863e34b5528416b53422c903437c1b6bb591148",
        ),
        (
            "Blizzard_Collections/Mainline/Blizzard_ToyBox.lua",
            "1fade6de5e41c5e605d8b0c64d7d76d3295a6bf1400566131816a3280a373cd9",
        ),
    ] {
        let bytes = std::fs::read(ui.join(relative)).expect("pinned original source");
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), digest, "{relative}");
    }
}

fn load_original_collections() -> WowLuaEnv {
    assert_eq!(ACTIVE, ClientProfile::WowForever);
    let ui = blizzard_ui_dir();
    assert_source_hashes(&ui);
    let toc =
        TocFile::from_file(&ui.join("Blizzard_Collections/Blizzard_Collections.toc")).unwrap();
    assert_selected_files(&toc);
    let selected = discover_blizzard_addon_closure_for_screen(
        &ui,
        ScreenKind::Game,
        &["Blizzard_Collections"],
    );
    assert!(
        selected
            .iter()
            .any(|(name, _)| name == "Blizzard_Collections")
    );
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_Collections"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_Collections"));
    env.exec(ORIGINAL_SOURCE)
        .expect("original selected Collections and ToyBox functions");
    env
}

fn supply_distinct_item_cooldowns(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.item_cooldowns.clear();
    sim.item_cooldowns.insert(
        6948,
        SpellCooldownState {
            start: 11.0,
            duration: 31.0,
        },
    );
    sim.item_cooldowns.insert(
        140192,
        SpellCooldownState {
            start: 17.0,
            duration: 43.0,
        },
    );
    assert!(!sim.item_cooldowns.contains_key(&880001));
}

#[test]
fn forever_original_collections_cooldown_observes_two_records_and_absence_without_errors() {
    let env = load_original_collections();
    supply_distinct_item_cooldowns(&env);
    // Loading the complete LoD closure may have unrelated baseline errors. Preserve
    // them and prove THIS original consumer path adds no caught or uncaught errors.
    let before = {
        let sim = env.state().borrow();
        (
            sim.lua_errors.clone(),
            sim.lua_error_records.len(),
            sim.lua_error_counts.clone(),
        )
    };
    env.exec(OBSERVE_COOLDOWNS)
        .expect("original consumer succeeds for two records and absence");
    let sim = env.state().borrow();
    assert_eq!(sim.lua_errors, before.0, "consumer emitted Lua errors");
    assert_eq!(
        sim.lua_error_records.len(),
        before.1,
        "consumer recorded Lua errors"
    );
    assert_eq!(
        sim.lua_error_counts, before.2,
        "consumer increased Lua error counts"
    );
}
