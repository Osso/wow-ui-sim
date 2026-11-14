//! Regression checks for profile publication through 70707's original Game consumers.
#![cfg(all(feature = "client-wowforever", feature = "optional-tests"))]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::{WowLuaEnv, state::AchievementInfo};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn startup(consumers: &[(&str, &str)]) -> WowLuaEnv {
    let root = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let addons = discover_blizzard_addons_for_screen(&root, ScreenKind::Game);
    for (name, file) in consumers {
        assert_selected(&addons, name, file);
    }

    let env = WowLuaEnv::new().unwrap();
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![root];
    wow_ui_sim::xml::register_intrinsic_templates();
    for (name, toc) in addons {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|e| panic!("load {name}: {e}"));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn assert_selected(addons: &[(String, PathBuf)], addon: &str, source: &str) {
    let (_, toc_path) = addons
        .iter()
        .find(|(name, _)| name == addon)
        .unwrap_or_else(|| panic!("{addon} must be a selected Game addon"));
    let toc = TocFile::from_file(toc_path).unwrap();
    assert!(
        toc.files.iter().any(|file| file == Path::new(source)),
        "Game TOC must select {addon}/{source}"
    );
}

// PRESERVATION: Camelot does not consume the award record. The original
// delegates depend on publication, so a constant callable could pass.
#[test]
fn forever_original_pvp_indicators_load_with_the_camelot_style() {
    let env = startup(&[
        (
            "Blizzard_UnitFrame",
            "Camelot/UnitFramePvPIndicatorStyle.lua",
        ),
        ("Blizzard_UnitFrame", "Shared/UnitFrameUtil.lua"),
        ("Blizzard_UnitFrame", "Mainline/PlayerFrame.lua"),
        ("Blizzard_UnitFrame", "Mainline/TargetFrame.lua"),
    ]);
    env.exec(
        r#"
        local info = PlayerFrame_UpdatePvPStatus()
        assert(type(info.showPvPIcon) == "boolean")
        assert(type(UnitFrameUtil.UpdateUnitPvPIndicator) == "function")
        assert(info.showPrestigePortrait == false)
        "#,
    )
    .expect("original Camelot style must provide the player and target indicator delegates");
}

// PRESERVATION: this single #948 snapshot proves original tracker wiring,
// but a constant {948} also passes without reading the tracked list.
#[test]
fn forever_original_achievement_ui_and_tracker_read_supplied_ids() {
    let env = startup(&[
        (
            "Blizzard_AchievementUI",
            "Mainline/Blizzard_AchievementUI.lua",
        ),
        (
            "Blizzard_ObjectiveTracker",
            "Blizzard_AchievementObjectiveTracker.lua",
        ),
    ]);
    let kind: i32 = env
        .eval("return Enum.ContentTrackingType.Achievement")
        .unwrap();
    {
        // Reuse the earned-reputation achievement and real criteria data
        // from achievements_api. The independent single-criteria eligibility
        // provider is not modeled and is not reached in stock startup.
        let mut sim = env.state().borrow_mut();
        sim.achievements.insert(
            948,
            AchievementInfo {
                achievement_id: 948,
                name: "Ambassador of the Alliance".into(),
                points: 10,
                description: "Earn exalted reputation with alliance cities.".into(),
                flags: 0,
                icon: 0,
                reward_text: String::new(),
                is_guild: false,
                is_statistic: false,
                reward_item_id: None,
            },
        );
        sim.content_tracking
            .tracked_ids_by_type
            .insert(u32::try_from(kind).unwrap(), vec![948]);
    }
    env.exec(
        r#"
        AchievementFrameAchievements_UpdateTrackedAchievements()
        AchievementObjectiveTracker:LayoutContents()
        assert(GetAchievementNumCriteria(948) > 1)
        assert(AchievementObjectiveTracker:GetExistingBlock(948) ~= nil)
        "#,
    )
    .expect("original AchievementUI and tracker must read the supplied tracked row");
}

// PRESERVATION: the unavailable-manager branch tests captured publication
// and initialization, not an enabled result; a constant false would pass.
#[test]
fn forever_original_combat_audio_manager_completes_startup_without_manager() {
    let env = startup(&[(
        "Blizzard_CombatAudioAlerts",
        "Blizzard_CombatAudioAlertManager.lua",
    )]);
    {
        let sim = env.state().borrow();
        assert!(sim.sound_manager.is_none());
        sim.cvars.set_imported("CAAEnabled", "1");
    }
    env.exec(
        r#"
        CombatAudioAlertManager:Init(true)
        assert(CombatAudioAlertManager.initDone)
        assert(not CombatAudioAlertManager:IsEventRegistered("PLAYER_IN_COMBAT_CHANGED"))
        "#,
    )
    .expect("original manager must honor unavailable audio backing without aborting startup");
}

// PRODUCER-SENSITIVE: the same original party caster yields different
// results for two supplied spell rows. A constant triple cannot pass.
#[test]
fn forever_original_aura_util_uses_supplied_visibility() {
    let env = startup(&[("Blizzard_FrameXMLUtil", "AuraUtil.lua")]);
    let (in_combat, out_of_combat): (i32, i32) = env
        .eval("return Enum.SpellAuraVisibilityType.RaidInCombat, Enum.SpellAuraVisibilityType.RaidOutOfCombat")
        .unwrap();
    let mut visibility = HashMap::new();
    for selector in [in_combat, out_of_combat] {
        let selector = u8::try_from(selector).unwrap();
        visibility.insert((19750, selector), (true, false, true));
        visibility.insert((642, selector), (true, true, false));
    }
    env.state().borrow_mut().spell_visibility_results = Some(visibility);
    env.exec(
        r#"
        assert(AuraUtil.ShouldDisplayDebuff("party1", 19750) == true)
        assert(AuraUtil.ShouldDisplayDebuff("party1", 642) == false)
        assert(AuraUtil.ShouldDisplayDebuff("player", 642) == true)
        "#,
    )
    .expect("original AuraUtil must consume the distinct host visibility rows");
}

// PRODUCER-SENSITIVE API-level backing check, not original-consumer RED: Camelot's original UnitFrame only observes
// publication, not the populated reward. Its style disables prestige and
// PVPMatchTableOverrides.lua replaces the shared honor cell's texture lookup.
#[test]
fn forever_honor_reward_query_projects_supplied_rows() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        for (level, badge) in [(2, 202), (6, 606)] {
            sim.pvp_catalog.honor_rewards.insert(
                level,
                wow_ui_sim::c_api::c_pvp::catalog::HonorRewardInfo {
                    badge_file_data_id: badge,
                    ..Default::default()
                },
            );
        }
    }
    env.exec(
        r#"
        assert(C_PvP.GetHonorRewardInfo(2).badgeFileDataID == 202)
        assert(C_PvP.GetHonorRewardInfo(6).badgeFileDataID == 606)
        "#,
    )
    .expect("the shared honor query must project actual supplied rows");
}
