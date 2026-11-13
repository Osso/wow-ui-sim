//! Generated-documentation contracts and existing-state projections for M4.
#![cfg(not(feature = "client-wrath"))]

use wow_ui_sim::lua_api::WowLuaEnv;

macro_rules! contract {
    ($name:ident, $lua:literal) => {
        #[test]
        fn $name() {
            WowLuaEnv::new().unwrap().exec($lua).unwrap();
        }
    };
}

contract!(
    combat_audio_enablement_is_explicitly_inactive,
    "assert(C_CombatAudioAlert.IsEnabled() == false)"
);

#[test]
fn combat_audio_throttles_reuse_the_setting_store() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_CombatAudioAlert.GetThrottle(0) == 0); assert(C_CombatAudioAlert.GetThrottle(1) == 0)").unwrap();
    #[cfg(feature = "retail-12-0-0")]
    {
        env.exec("assert(C_CombatAudioAlert.SetThrottle(0, 1.25)); assert(C_CombatAudioAlert.GetThrottle(0) == 1.25); assert(C_CombatAudioAlert.GetThrottle(1) == 0)").unwrap();
    }
}

#[cfg(not(feature = "retail-12-0-0"))]
contract!(
    throttle_read_does_not_publish_an_unrouted_setter,
    "assert(C_CombatAudioAlert.SetThrottle == nil)"
);

#[cfg(any(
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
mod classic {
    use super::*;
    contract!(
        map_highlight_has_no_polygon_snapshot,
        "assert(select('#', C_Map.GetMapHighlightInfoAtPosition(1, 0.5, 0.5)) == 0)"
    );
    contract!(
        world_pvp_queue_has_no_wait_estimate,
        "assert(C_PvP.GetWorldPvPWaitTime(1) == 0)"
    );
    contract!(
        quest_reward_spell_ids_are_not_inferred_from_legacy_names,
        "local spells = C_QuestInfoSystem.GetQuestRewardSpells(); assert(type(spells) == 'table' and next(spells) == nil)"
    );
}

#[cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
mod modern {
    use super::*;
    contract!(
        super_tracking_has_no_vignette_snapshot,
        "assert(C_SuperTrack.GetSuperTrackedVignette() == nil); assert(C_SuperTrack.IsSuperTrackingAnything() == false)"
    );
    contract!(
        hybrid_minimap_is_explicitly_disabled,
        "assert(C_Minimap.ShouldUseHybridMinimap() == false)"
    );
    #[cfg(feature = "client-retail")]
    contract!(
        honor_reward_catalog_is_explicitly_unavailable,
        "assert(C_PvP.GetHonorRewardInfo(1) == nil)"
    );
    #[cfg(feature = "client-retail")]
    contract!(
        landing_page_unlock_state_is_explicitly_unavailable,
        "assert(C_PlayerInfo.IsExpansionLandingPageUnlockedForPlayer(10) == false)"
    );
    contract!(
        quest_calling_membership_is_explicitly_unavailable,
        "assert(C_QuestLog.IsQuestCalling(990001) == false)"
    );

    #[test]
    fn pvp_match_query_tracks_existing_state() {
        let env = WowLuaEnv::new().unwrap();
        env.exec("assert(C_PvP.IsMatchActive() == false)").unwrap();
        env.state().borrow_mut().pvp_match_active = true;
        env.exec("assert(C_PvP.IsMatchActive() == true)").unwrap();
    }

    #[test]
    fn original_aura_container_handle_can_register_and_unregister() {
        let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
        let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
            &cache,
            &["Blizzard_AuraContainer"],
            &[],
        );
        assert!(loaded.iter().any(|name| name == "Blizzard_AuraContainer"));
        env.exec(
            r#"
            local source = assert(__secureenv.AuraContainerPrivateMixin)
            assert(type(source.OnLoad_Intrinsic) == 'function', 'secure AuraContainer OnLoad unavailable')
            local frame = Mixin(CreateFrame('Frame'), source)
            frame.unitToken = 'player'
            local received
            frame.OnUnitPrivateAuraUpdate = function(_, unit, info)
                assert(unit == 'player', 'original callback unit mismatch'); received = info
            end
            frame:OnLoad_Intrinsic()
            frame.privateAurasUpdateCallback:Register('player')
            C_UnitAurasPrivate._TriggerPrivateAuraUpdate('player', 'Private', {isFullUpdate=true})
            assert(received.isFullUpdate == true, 'original callback did not receive updateInfo')
            frame.privateAurasUpdateCallback:Unregister()
            frame.privateAurasUpdateCallback:Unregister()
            assert(C_UnitAurasPrivate._TriggerPrivateAuraUpdate('player', 'Private', {}) == 0)
        "#,
        )
        .unwrap();
    }

    #[test]
    fn private_aura_callback_removal_preserves_other_units_and_containers() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            r#"
            local first, second = 0, 0
            local callback = C_FunctionContainers.CreateCallback(function(info)
                first = first + info.amount
            end)
            local other = C_FunctionContainers.CreateCallback(function(info)
                second = second + info.amount
            end)
            C_UnitAurasPrivate.AddPrivateAuraUpdateCallback('player', callback)
            C_UnitAurasPrivate.AddPrivateAuraUpdateCallback('player', other)
            C_UnitAurasPrivate.AddPrivateAuraUpdateCallback('target', callback)
            C_UnitAurasPrivate.RemovePrivateAuraUpdateCallback('player', callback)
            collectgarbage('collect')
            assert(C_UnitAurasPrivate._TriggerPrivateAuraUpdate('player', {}, {amount=2}) == 1)
            assert(first == 0 and second == 2)
            assert(C_UnitAurasPrivate._TriggerPrivateAuraUpdate('target', {}, {amount=3}) == 1)
            assert(first == 3 and second == 2)
            C_UnitAurasPrivate.RemovePrivateAuraUpdateCallback('player', callback)
            C_UnitAurasPrivate.RemovePrivateAuraUpdateCallback('target', callback)
            assert(C_UnitAurasPrivate._TriggerPrivateAuraUpdate('target', {}, {amount=5}) == 0)
        "#,
        )
        .unwrap();
    }

    contract!(
        private_raid_boss_callbacks_do_not_dispatch_plain_functions,
        r#"
        local called = false
        C_UnitAurasPrivate.SetPrivateRaidBossMessageCallback(function() called = true end)
        local ok = pcall(C_UnitAurasPrivate._TriggerPrivateRaidBossMessage, 'Warning', 'RAID_BOSS_EMOTE')
        assert(not ok, 'plain-function callbacks must not be dispatched')
        assert(not called, 'unsupported callback was invoked')
        "#
    );

    contract!(
        private_raid_boss_message_callback_replaces_and_clears,
        r#"
        local old, current = 0, 0
        C_UnitAurasPrivate.SetPrivateRaidBossMessageCallback(C_FunctionContainers.CreateCallback(function() old = old + 1 end))
        C_UnitAurasPrivate.SetPrivateRaidBossMessageCallback(C_FunctionContainers.CreateCallback(function(message, kind)
            assert(message == 'Warning' and kind == 'RAID_BOSS_EMOTE'); current = current + 1
        end))
        collectgarbage('collect')
        C_UnitAurasPrivate._TriggerPrivateRaidBossMessage('Warning', 'RAID_BOSS_EMOTE')
        assert(old == 0 and current == 1)
        C_UnitAurasPrivate.SetPrivateRaidBossMessageCallback(nil)
        C_UnitAurasPrivate._TriggerPrivateRaidBossMessage('Warning', 'RAID_BOSS_EMOTE')
        assert(current == 1)
        "#
    );
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_objectives_project_existing_supplied_progress() {
    use wow_ui_sim::c_api::quest_objectives::{QuestObjective, QuestObjectiveProgress};
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .quest_log_entries
        .objective_progress
        .insert(
            990001,
            QuestObjectiveProgress {
                objectives: vec![QuestObjective {
                    text: "Wolves: 1/3".into(),
                    objective_type: "monster".into(),
                    finished: false,
                    num_fulfilled: 1,
                    num_required: 3,
                    criteria_id: None,
                }],
                ..Default::default()
            },
        );
    env.exec("local rows = C_QuestLog.GetQuestObjectives(990001); assert(#rows == 1 and rows[1].numFulfilled == 1 and rows[1].numRequired == 3); assert(C_QuestLog.GetQuestObjectives(990002) == nil)").unwrap();
}

#[cfg(feature = "client-wowforever")]
mod forever {
    use super::*;
    contract!(
        cvars_are_loaded_before_the_lua_environment_is_published,
        "assert(C_CVar.AreCVarsLoaded() == true)"
    );
    contract!(
        hidden_group_buffs_project_supplied_snapshot,
        "C_UnitAuras._hiddenGroupBuffs = {456}; local ids = C_UnitAuras.GetHiddenGroupBuffs(); assert(#ids == 1 and ids[1] == 456)"
    );
    contract!(
        hidden_buff_publication_does_not_enable_unrouted_visual_alert_apis,
        "assert(C_UnitAuras.GetGroupBuffVisualAlerts == nil); assert(C_UnitAuras.SetGroupBuffVisualAlerts == nil); assert(C_UnitAuras.SetHiddenGroupBuffs == nil)"
    );
    contract!(
        identity_secrecy_accepts_party_token_without_party,
        "assert(type(C_Secrets.ShouldUnitIdentityBeSecret) == 'function'); C_Secrets.ShouldUnitIdentityBeSecret('party1')"
    );
    contract!(
        invitation_candidates_validates_required_club_id,
        r#"
        local getCandidates = C_Club.GetInvitationCandidates
        assert(type(getCandidates) == 'function')
        local ok, message = pcall(getCandidates)
        local usage = 'Usage: local candidates = C_Club.GetInvitationCandidates([filter, maxResults, cursorPosition, allowFullMatch], clubId)'
        assert(not ok and string.find(message, usage, 1, true), tostring(message))
        ok, message = pcall(getCandidates, nil, nil, nil, nil, {})
        assert(not ok and string.find(message, usage, 1, true), tostring(message))
        "#
    );
    #[cfg(feature = "optional-tests")]
    #[test]
    fn invitation_candidates_return_inferred_empty_table() {
        let env = WowLuaEnv::new().unwrap();
        env.exec(
            "local rows = C_Club.GetInvitationCandidates(nil, nil, nil, nil, 'unknown-club'); assert(type(rows) == 'table' and next(rows) == nil)",
        )
        .unwrap();
    }
    contract!(
        can_trade_skill_show_crafting_ui_is_registered,
        "assert(type(C_TradeSkillUI.CanTradeSkillShowCraftingUI) == 'function')"
    );
    contract!(
        hardcore_active_returns_false_when_rule_10_is_unset,
        "assert(type(C_GameRules.IsHardcoreActive) == 'function'); assert(C_GameRules.IsHardcoreActive() == false)"
    );
    #[cfg(feature = "optional-tests")]
    #[test]
    fn hardcore_active_reads_nonzero_native_rule_10() {
        let env = WowLuaEnv::new().unwrap();
        env.state()
            .borrow_mut()
            .game_rules
            .rules_by_id
            .insert(10, 1);
        env.exec("assert(C_GameRules.IsHardcoreActive() == true)")
            .unwrap();
    }
    contract!(
        experience_preset_is_not_guessed,
        "assert(C_GameRules.GetForeverExperiencePreset() == nil)"
    );
    contract!(
        quest_timer_service_has_no_timers,
        "local timers = C_QuestLog.GetQuestTimers(); assert(type(timers) == 'table' and next(timers) == nil)"
    );
}
