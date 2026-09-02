use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn lua_api_globals_temporary_shims_module_is_removed() {
    assert!(
        !std::path::Path::new("src/lua_api/globals/temporary_shims/mod.rs").exists(),
        "temporary Lua compatibility defaults should register directly from missing_surface or lua_api::workarounds, not through globals::temporary_shims"
    );
}

#[test]
fn modeled_c_lfg_info_reports_inactive_unknown_follower_dungeon() {
    let env = env();
    let is_follower: bool = env
        .eval("return C_LFGInfo.IsLFGFollowerDungeon(999999)")
        .unwrap();
    assert!(
        !is_follower,
        "an unknown dungeon should not be classified as a follower dungeon"
    );
}

#[test]
fn modeled_c_death_recap_lives_in_c_api() {
    assert!(
        !std::path::Path::new("src/lua_api/globals/missing_surface/death_recap.rs").exists(),
        "modeled C_DeathRecap state belongs under src/c_api, not lua_api::globals::missing_surface"
    );
    assert!(
        std::path::Path::new("src/c_api/c_death_recap.rs").exists(),
        "C_DeathRecap should have an explicit C API owner"
    );
}

#[test]
fn modeled_c_social_returns_friend_status_defaults() {
    let env = env();
    let (afk, dnd, raf_link_type): (bool, bool, i32) = env
        .eval(
            r#"
            local friend = C_Social.GetFriendInfo(1)
            return friend.afk, friend.dnd, friend.rafLinkType
            "#,
        )
        .unwrap();
    assert!(!afk, "seeded friends should not be AFK");
    assert!(!dnd, "seeded friends should not be DND");
    assert_eq!(raf_link_type, 0, "seeded friends use the default RAF link type");
}

#[test]
fn modeled_c_chat_bubbles_returns_empty_default_collection() {
    let env = env();
    let (count, first_is_nil): (i32, bool) = env
        .eval(
            r#"
            local bubbles = C_ChatBubbles.GetAllChatBubbles()
            return #bubbles, bubbles[1] == nil
            "#,
        )
        .unwrap();
    assert_eq!(count, 0, "the default simulator state has no chat bubbles");
    assert!(first_is_nil, "the empty bubble collection has no first entry");
}

#[test]
fn modeled_c_party_info_has_no_active_group_by_default() {
    let env = env();
    let group_type: Option<i32> = env.eval("return C_PartyInfo.GetActiveGroupType()").unwrap();
    assert!(
        group_type.is_none(),
        "the default simulator state has no active party or raid"
    );
}

#[test]
fn modeled_c_character_services_have_no_active_boosts_by_default() {
    let env = env();
    let (upgrade, trial): (Option<i32>, Option<i32>) = env
        .eval(
            "return C_CharacterServices.GetActiveCharacterUpgradeBoostType(), C_CharacterServices.GetActiveClassTrialBoostType()",
        )
        .unwrap();
    assert!(upgrade.is_none(), "no character upgrade boost is active by default");
    assert!(trial.is_none(), "no class trial boost is active by default");
}

#[test]
fn modeled_c_report_system_allocates_positive_report_token() {
    let env = env();
    let token: i64 = env
        .eval("return C_ReportSystem.InitiateReportPlayer('cheating')")
        .unwrap();
    assert!(token > 0, "a new player report should receive a positive token");
}

#[test]
fn modeled_c_summon_info_exposes_inactive_request_shape() {
    let env = env();
    let (reason_missing, time_left, skips_start): (bool, i32, bool) = env
        .eval(
            r#"
            return C_SummonInfo.GetSummonReason() == nil,
                   C_SummonInfo.GetSummonConfirmTimeLeft(),
                   C_SummonInfo.IsSummonSkippingStartExperience()
            "#,
        )
        .unwrap();
    assert!(reason_missing, "an inactive summon has no reason");
    assert_eq!(time_left, 0, "an inactive summon has no remaining confirmation time");
    assert!(
        !skips_start,
        "an inactive summon does not skip the starting experience"
    );
}

#[test]
fn modeled_c_stable_info_reports_closed_stable_by_default() {
    let env = env();
    let at_pet_stable: bool = env.eval("return C_StableInfo.IsAtPetStable()").unwrap();
    assert!(
        !at_pet_stable,
        "the default simulator state is not at a pet stable"
    );
}

#[test]
fn modeled_c_account_services_returns_disabled_save_result() {
    let env = env();
    let (started, result_code, locked): (bool, i32, bool) = env
        .eval(
            r#"
            local started, resultCode = C_AccountServices.SaveAccountData()
            return started, resultCode, C_AccountServices.IsAccountLockedPostSave()
            "#,
        )
        .unwrap();
    assert!(!started, "saving is disabled in the default simulator state");
    assert_eq!(result_code, 10, "a disabled save returns the unavailable result code");
    assert!(!locked, "a disabled save does not lock the account");
}

#[test]
fn modeled_c_battle_net_reports_enabled_retail_friend_features() {
    let env = env();
    let (friend_tags_enabled, list_supported): (bool, bool) = env
        .eval(
            "return C_BattleNet.AreFriendTagsEnabled(), C_BattleNet.IsBattleNetFriendsListSupported()",
        )
        .unwrap();
    assert!(friend_tags_enabled, "retail friend tags should be enabled");
    assert!(list_supported, "the Battle.net friend list should be supported");
}
