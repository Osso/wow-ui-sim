//! Smoke tests for startup-surface stubs added to unblock Blizzard addon loading.

#[path = "startup_api_stubs/common.rs"]
mod startup_api_common;

use startup_api_common::*;

type GlueRuntimeProbe = (
    String,
    bool,
    bool,
    String,
    bool,
    String,
    String,
    bool,
    f64,
    String,
    String,
    bool,
    bool,
    bool,
    String,
    f64,
    String,
    bool,
    bool,
);

type GlueCharacterSelectProbe = (
    bool,
    bool,
    bool,
    f64,
    String,
    f64,
    f64,
    bool,
    String,
    String,
    f64,
    bool,
    bool,
);

#[test]
fn glue_runtime_helpers_exist_with_safe_defaults() {
    let env = env();
    let (
        c_ui_type,
        avoids_notch,
        has_display_notch,
        c_glue_type,
        first_load,
        saved_account_name,
        saved_account_list,
        screen_first_displayed,
        login_background,
        min_expansion_level_type,
        server_name_type,
        player_location_kind,
        player_location_valid,
        adventure_guide_available,
        dungeon_normal_name,
        dungeon_normal_max_players,
        player_spells_util_type,
        has_spellbook_toggle,
        has_talent_toggle,
    ): GlueRuntimeProbe = env
        .eval(
            r#"
            UnitIsHumanPlayer = function(unit)
                return unit == "player"
            end

            local saved = GetSavedAccountList()
            local location = PlayerLocation:CreateFromUnit("player")

            return type(C_UI),
                   C_UI.ShouldUIParentAvoidNotch(),
                   C_UI.DoesAnyDisplayHaveNotch(),
                   type(C_Glue),
                   C_Glue.IsFirstLoadThisSession(),
                   GetSavedAccountName(),
                   saved,
                   WasScreenFirstDisplayed("login"),
                   GetLoginScreenBackground(42, 7),
                   type(GetMinimumExpansionLevel()),
                   type(GetServerName()),
                   location:IsUnit(),
                   location:IsValid(),
                   AdventureGuideUtil.IsAvailable(),
                   DifficultyUtil.GetDifficultyName(DifficultyUtil.ID.DungeonNormal),
                   DifficultyUtil.GetMaxPlayers(DifficultyUtil.ID.DungeonNormal),
                   type(PlayerSpellsUtil),
                   type(PlayerSpellsUtil.ToggleSpellBookFrame) == "function",
                   type(PlayerSpellsUtil.ToggleClassTalentFrame) == "function"
            "#,
        )
        .expect("glue runtime helpers should be callable");

    assert_eq!(c_ui_type, "table");
    assert!(!avoids_notch);
    assert!(!has_display_notch);
    assert_eq!(c_glue_type, "table");
    assert!(!first_load);
    assert_eq!(saved_account_name, "");
    assert_eq!(saved_account_list, "");
    assert!(!screen_first_displayed);
    assert_eq!(login_background, 42.0);
    assert_eq!(min_expansion_level_type, "number");
    assert_eq!(server_name_type, "nil");
    assert!(player_location_kind);
    assert!(player_location_valid);
    assert!(adventure_guide_available);
    assert!(!dungeon_normal_name.is_empty());
    assert_eq!(dungeon_normal_max_players, 5.0);
    assert_eq!(player_spells_util_type, "table");
    assert!(has_spellbook_toggle);
    assert!(has_talent_toggle);
}

#[test]
fn glue_character_select_helpers_exist_with_safe_defaults() {
    let env = env();
    let (
        set_model_frame_ok,
        set_map_scene_ok,
        set_world_frame_ok,
        max_groups,
        timerunning_season_kind,
        min_render_scale,
        max_render_scale,
        expansion_trial,
        recruit_active_type,
        recruit_faction_type,
        upgrade_expansion_level,
        undelete_enabled,
        undelete_cooldown,
    ): GlueCharacterSelectProbe = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "CodexGlueWorldFrame", UIParent)
            local setWorldFrameOK = pcall(function()
                SetWorldFrameStrata(frame)
            end)
            local setModelFrameOK = pcall(function()
                SetCharSelectModelFrame("ModelFFX")
            end)
            local setMapSceneOK = pcall(function()
                SetCharSelectMapSceneFrame("MapScene")
            end)

            return setModelFrameOK,
                   setMapSceneOK,
                   setWorldFrameOK,
                   GetMaxWarbandGroupCount(),
                   type(GetActiveTimerunningSeasonID()),
                   GetMinRenderScale(),
                   GetMaxRenderScale(),
                   IsExpansionTrial(),
                   type(select(1, C_RecruitAFriend.GetRecruitInfo())),
                   type(select(2, C_RecruitAFriend.GetRecruitInfo())),
                   GetUpgradeExpansionLevel(),
                   GetCharacterUndeleteStatus()
            "#,
        )
        .expect("glue character-select helpers should be callable");

    assert!(set_model_frame_ok);
    assert!(set_map_scene_ok);
    assert!(set_world_frame_ok);
    assert_eq!(max_groups, 4.0);
    assert_eq!(timerunning_season_kind, "nil");
    assert_eq!(min_render_scale, 0.5);
    assert_eq!(max_render_scale, 1.0);
    assert!(!expansion_trial);
    assert_eq!(recruit_active_type, "boolean");
    assert_eq!(recruit_faction_type, "nil");
    assert_eq!(upgrade_expansion_level, 80.0);
    assert!(!undelete_enabled);
    assert!(!undelete_cooldown);
}

#[test]
fn c_lfg_info_can_player_use_premade_group_can_be_disabled() {
    let env = env();
    env.exec("A_Admin.SetCanUsePremadeGroup(false)").unwrap();
    let can_use: bool = env
        .eval("return C_LFGInfo.CanPlayerUsePremadeGroup()")
        .unwrap();
    assert!(
        !can_use,
        "premade group finder is not simulated, so the callsite takes the \
         'cannot use' branch and skips the premade promo UI"
    );
}

#[test]
fn recruit_a_friend_surface_returns_disabled_empty_defaults() {
    let env = env();
    let (
        enabled_probe,
        recruiting_enabled,
        versions_len,
        recruits_len,
        claim_in_progress,
        recruit_active,
        recruit_faction_type,
    ): (String, bool, f64, f64, bool, bool, String) = env
        .eval(
            r#"
            local info = C_RecruitAFriend.GetRAFInfo()
            local active, faction = C_RecruitAFriend.GetRecruitInfo()
            local enabled = C_RecruitAFriend.IsEnabled
            return enabled and tostring(enabled()) or "absent",
                   C_RecruitAFriend.IsRecruitingEnabled(),
                   #info.versions,
                   #info.recruits,
                   info.claimInProgress,
                   active,
                   type(faction)
            "#,
        )
        .expect("Recruit-A-Friend fallback surface should be callable");
    // 12.1.0 removed C_RecruitAFriend.IsEnabled.
    let expected_enabled = if cfg!(feature = "retail-12-1-0") {
        "absent"
    } else {
        "false"
    };
    assert_eq!(enabled_probe, expected_enabled);
    assert!(!recruiting_enabled);
    assert_eq!(versions_len, 1.0);
    assert_eq!(recruits_len, 0.0);
    assert!(!claim_in_progress);
    assert!(!recruit_active);
    assert_eq!(recruit_faction_type, "nil");
}
