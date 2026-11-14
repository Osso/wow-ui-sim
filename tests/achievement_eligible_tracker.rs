//! R88o.1: test the original selected tracker and its actual objective font string.
#![cfg(all(
    feature = "optional-tests",
    any(
        feature = "client-retail",
        feature = "client-ptr",
        feature = "client-wowforever"
    )
))]

use std::path::Path;
use wow_ui_sim::lua_api::{
    WowLuaEnv,
    state::{
        AchievementEligibilityCriterion, AchievementEligibilityNode,
        AchievementEligibilityOperation,
    },
};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn original_tracker() -> WowLuaEnv {
    let root = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let addons = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&root, ScreenKind::Game);
    let (_, toc_path) = addons
        .iter()
        .find(|(name, _)| name == "Blizzard_ObjectiveTracker")
        .expect("ObjectiveTracker in selected Game addons");
    let toc = TocFile::from_file(toc_path).unwrap();
    assert!(
        toc.files
            .iter()
            .any(|file| file == Path::new("Blizzard_AchievementObjectiveTracker.lua"))
    );

    let env = WowLuaEnv::new().expect("Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![root];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
            .unwrap_or_else(|err| panic!("load {name}: {err}"));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    let kind: i32 = env
        .eval("return Enum.ContentTrackingType.Achievement")
        .unwrap();
    {
        let mut sim = env.state().borrow_mut();
        assert!(
            sim.achievements.contains_key(&6),
            "catalog-backed Level 10 fixture"
        );
        sim.content_tracking
            .tracked_ids_by_type
            .insert(u32::try_from(kind).unwrap(), vec![6]);
    }
    env
}

fn observe(env: &WowLuaEnv, style: &str) {
    env.exec(&format!(
            r#"
            local tracker = assert(AchievementObjectiveTracker, "original XML tracker")
            local source = debug.getinfo(tracker.LayoutContents, "S").source
            assert(source:find("Blizzard_AchievementObjectiveTracker.lua", 1, true), source)
            assert(GetAchievementNumCriteria(6) == 0, "exercise original single/zero criteria branch")
            -- Run the real module layout lifecycle with a positive viewport budget.
            tracker:Update(400, false)
            local block = assert(tracker:GetExistingBlock(6), "host-tracked objective was not rendered")
            assert(block:IsShown(), "objective block was not shown by original layout")
            local line = assert(block.usedLines[1], "original block has no objective row")
            local description = select(8, GetAchievementInfo(6))
            assert(type(description) == "string" and description ~= "", "missing catalog description")
            assert(line.Text:GetText():find(description, 1, true), "wrong objective text")
            local expected = OBJECTIVE_TRACKER_COLOR["{style}"]
            local actual = line.Text.colorStyle
            assert(actual == expected or actual == expected.reverse, "wrong eligibility color style")
            local r, g, b = line.Text:GetTextColor()
            assert(math.abs(r - actual.r) < 0.001 and math.abs(g - actual.g) < 0.001
                and math.abs(b - actual.b) < 0.001, "actual font string color differs")
            "#,
        ))
    .unwrap_or_else(|err| panic!("original tracker should render {style}: {err}"));
}

const TREE_ID: i32 = 99_006;
const CRITERION_ID: i32 = 99_007;
const WORLD_STATE_ID: i32 = 99_008;

#[test]
fn original_achievement_tracker_distinguishes_missing_group_from_null_tree() {
    let env = original_tracker();
    {
        let mut sim = env.state().borrow_mut();
        sim.achievement_eligibility
            .achievement_tree_ids
            .insert(6, TREE_ID);
    }
    observe(&env, "Normal");
    env.state()
        .borrow_mut()
        .achievement_eligibility
        .groups
        .insert(TREE_ID, None);
    observe(&env, "Failed");
}

#[test]
fn original_achievement_tracker_colors_world_state_leaf_mismatch() {
    let env = original_tracker();
    {
        let mut sim = env.state().borrow_mut();
        let eligibility = &mut sim.achievement_eligibility;
        eligibility.achievement_tree_ids.insert(6, TREE_ID);
        eligibility.groups.insert(
            TREE_ID,
            Some(AchievementEligibilityNode {
                id: TREE_ID,
                children: vec![],
            }),
        );
        eligibility.criteria_tree.insert(
            TREE_ID,
            AchievementEligibilityOperation::Leaf {
                criterion_id: CRITERION_ID,
            },
        );
        eligibility.criteria.insert(
            CRITERION_ID,
            AchievementEligibilityCriterion {
                world_state_id: WORLD_STATE_ID,
                required_value: 7,
            },
        );
        eligibility.world_states.insert(WORLD_STATE_ID, 11);
    }
    observe(&env, "Failed");
    env.state()
        .borrow_mut()
        .achievement_eligibility
        .world_states
        .insert(WORLD_STATE_ID, 7);
    observe(&env, "Normal");
}
