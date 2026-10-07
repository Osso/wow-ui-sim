//! Integration tests for the legacy archaeology completion-history surface
//! (`IsArtifactCompletionHistoryAvailable`, `GetArtifactInfoByRace`,
//! `RequestArtifactCompletionHistory`) consumed by the completed-page
//! paginator at `Blizzard_ArchaeologyUI/Blizzard_ArchaeologyUI.lua:436`,
//! `:450`, and `:540`.

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{ArchaeologyArtifact, ArchaeologyRace};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

fn dwarf_artifact(name: &str, rarity: i32, completion_count: i32) -> ArchaeologyArtifact {
    ArchaeologyArtifact {
        name: name.to_string(),
        description: format!("{name} description"),
        rarity,
        icon: 134419,
        spell_description: format!("{name} spell text"),
        spell_id: 88910,
        first_completion_time: 1_700_000_000,
        completion_count,
    }
}

fn seed_history(env: &WowLuaEnv) {
    let state = env.state();
    let mut sim = state.borrow_mut();
    sim.archaeology.history_available = true;
    sim.archaeology.races = vec![ArchaeologyRace {
        name: "Dwarf".to_string(),
        texture: 460983,
        race_item_id: 63127,
        currency_amount: 0,
        project_amount: 35,
        artifacts: vec![
            dwarf_artifact("Belt Buckle of Zaldarinnu", 0, 3),
            dwarf_artifact("Bones of Transformation", 1, 1),
        ],
    }];
}

#[test]
fn is_artifact_completion_history_available_defaults_to_false() {
    let env = env();
    let result: bool = env
        .eval("return IsArtifactCompletionHistoryAvailable()")
        .unwrap();
    assert!(
        !result,
        "the paginator at :436 must hide every row until the server delivers history",
    );
}

#[test]
fn is_artifact_completion_history_available_reflects_seeded_state() {
    let env = env();
    seed_history(&env);
    let result: bool = env
        .eval("return IsArtifactCompletionHistoryAvailable()")
        .unwrap();
    assert!(result);
}

#[test]
#[cfg(not(any(feature = "client-retail", feature = "client-ptr")))]
fn request_artifact_completion_history_flips_availability_to_true() {
    let env = env();
    env.exec("RequestArtifactCompletionHistory()").unwrap();
    let st = env.state().borrow();
    assert!(
        st.archaeology.history_available,
        "the stub server response makes history immediately available so addons that wait on the request can proceed",
    );
}

#[test]
fn get_artifact_info_by_race_returns_nothing_for_out_of_range_race() {
    let env = env();
    seed_history(&env);
    let returned: Option<String> = env.eval("return (GetArtifactInfoByRace(99, 1))").unwrap();
    assert!(
        returned.is_none(),
        "the addon's `if not name` branch at :452 must trigger so it advances to the next race",
    );
}

#[test]
fn get_artifact_info_by_race_returns_nothing_past_last_project() {
    let env = env();
    seed_history(&env);
    let returned: Option<String> = env.eval("return (GetArtifactInfoByRace(1, 99))").unwrap();
    assert!(
        returned.is_none(),
        "running off the end of a race's artifact list must yield nil so the paginator stops",
    );
}

#[test]
fn get_artifact_info_by_race_returns_nothing_for_zero_indices() {
    let env = env();
    seed_history(&env);
    let zero_race: Option<String> = env.eval("return (GetArtifactInfoByRace(0, 1))").unwrap();
    let zero_project: Option<String> = env.eval("return (GetArtifactInfoByRace(1, 0))").unwrap();
    assert!(zero_race.is_none(), "raceIndex is 1-based; 0 must read nil");
    assert!(
        zero_project.is_none(),
        "projectIndex is 1-based; 0 must read nil",
    );
}

#[test]
fn get_artifact_info_by_race_returns_ten_values_for_valid_indices() {
    let env = env();
    seed_history(&env);
    let (
        name,
        description,
        rarity,
        icon,
        spell_description,
        unused_six,
        unused_seven,
        spell_id,
        first_completion_time,
        completion_count,
    ): (String, String, i32, i32, String, i32, i32, i32, f64, i32) =
        env.eval("return GetArtifactInfoByRace(1, 1)").unwrap();
    assert_eq!(name, "Belt Buckle of Zaldarinnu");
    assert_eq!(description, "Belt Buckle of Zaldarinnu description");
    assert_eq!(rarity, 0);
    assert_eq!(icon, 134419);
    assert_eq!(spell_description, "Belt Buckle of Zaldarinnu spell text");
    assert_eq!(
        unused_six, 0,
        "position 6 is documented unused — addon destructures with `_,` at :450",
    );
    assert_eq!(unused_seven, 0, "position 7 is documented unused");
    assert_eq!(spell_id, 88910);
    assert!(
        (first_completion_time - 1_700_000_000.0).abs() < 1.0,
        "firstCompletionTime is the seeded epoch",
    );
    assert_eq!(completion_count, 3);
}

#[test]
fn get_artifact_info_by_race_resolves_second_project() {
    let env = env();
    seed_history(&env);
    let (name, _, rarity, _, _, _, _, _, _, completion_count): (
        String,
        String,
        i32,
        i32,
        String,
        i32,
        i32,
        i32,
        f64,
        i32,
    ) = env.eval("return GetArtifactInfoByRace(1, 2)").unwrap();
    assert_eq!(name, "Bones of Transformation");
    assert_eq!(rarity, 1, "rare artifact rarity");
    assert_eq!(completion_count, 1);
}

#[test]
fn paginator_loop_stops_at_first_nil_per_race() {
    // Mirrors the per-race walk at Blizzard_ArchaeologyUI.lua:450 — the
    // addon increments projectIndex until the first nil, then advances to
    // the next race. We verify that the simulator's `nil` boundary is
    // visible to plain Lua iteration.
    let env = env();
    seed_history(&env);
    let count: i32 = env
        .eval(
            r#"
            local count = 0
            local i = 1
            while true do
                local name = GetArtifactInfoByRace(1, i)
                if not name then break end
                count = count + 1
                i = i + 1
            end
            return count
        "#,
        )
        .unwrap();
    assert_eq!(count, 2, "two seeded artifacts in race 1, then nil");
}
