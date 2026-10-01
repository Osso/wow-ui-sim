//! Pending actual-query fixtures for structures-RecentAllyStateData-669.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use wow_ui_sim::c_api::c_recent_allies::{
    RecentAllyCharacterData, RecentAllyData, RecentAllyInteraction,
    RecentAllyInteractionContextData, RecentAllyInteractionData, RecentAllyStateData,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_interactions() -> RecentAllyInteractionData {
    RecentAllyInteractionData {
        note: Some("Helped with the dungeon".into()),
        interactions: vec![
            RecentAllyInteraction {
                interaction_type: 2,
                description: "Completed fixture dungeon".into(),
                timestamp: 1_760_000_123,
                context_data: RecentAllyInteractionContextData {
                    item_id: Some(210001),
                    location_name: Some("Fixture dungeon".into()),
                    activity_difficulty_id: Some(8),
                    activity_difficulty_level: Some(12),
                },
            },
            RecentAllyInteraction {
                interaction_type: 3,
                description: "Traded supplies".into(),
                timestamp: 1_760_000_456,
                context_data: RecentAllyInteractionContextData {
                    item_id: None,
                    location_name: None,
                    activity_difficulty_id: None,
                    activity_difficulty_level: None,
                },
            },
        ],
    }
}

fn fixture_ally() -> RecentAllyData {
    RecentAllyData {
        state_data: RecentAllyStateData {
            is_online: true,
            is_dnd: false,
            is_afk: true,
            is_converted_legacy_friend: false,
            pin_expiration_date: Some(1_760_100_000),
            friend_request_sent_this_session: true,
            current_location: Some("Dornogal".into()),
        },
        character_data: RecentAllyCharacterData {
            guid: "Player-100-00000001".into(),
            name: "Aster".into(),
            full_name: "Aster-FixtureRealm".into(),
            realm_name: "FixtureRealm".into(),
            level: 80,
            class_id: 2,
            race_id: 1,
            sex: 3,
        },
        interaction_data: fixture_interactions(),
    }
}

fn populated_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Recent Allies environment");
    let first = fixture_ally();
    let mut second = first.clone();
    second.state_data = RecentAllyStateData {
        is_online: false,
        is_dnd: true,
        is_afk: false,
        is_converted_legacy_friend: true,
        pin_expiration_date: None,
        friend_request_sent_this_session: false,
        current_location: None,
    };
    second.character_data.guid = "Player-100-00000002".into();
    second.character_data.name = "Birch".into();
    second.character_data.full_name = "Birch-FixtureRealm".into();
    second.interaction_data = RecentAllyInteractionData {
        interactions: vec![],
        note: None,
    };
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state.recent_allies.enabled = true;
        state.recent_allies.entries = vec![first, second];
    }
    env
}

#[test]
fn recent_allies_disabled_returns_nothing_even_with_input_rows() {
    let env = populated_env();
    env.state().borrow_mut().recent_allies.enabled = false;
    env.exec(
        r##"
        assert(type(C_RecentAllies.GetRecentAllies) == "function")
        assert(select("#", C_RecentAllies.GetRecentAllies()) == 0,
            "disabled RequiresRecentAllies must return nothing")
        "##,
    )
    .unwrap();
}

#[test]
fn recent_allies_enabled_empty_input_returns_one_empty_table() {
    let env = WowLuaEnv::new().expect("Recent Allies environment");
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        assert!(!state.recent_allies.enabled);
        assert!(state.recent_allies.entries.is_empty());
        state.recent_allies.enabled = true;
    }
    env.exec(
        r##"
        assert(select("#", C_RecentAllies.GetRecentAllies()) == 1)
        local allies = C_RecentAllies.GetRecentAllies()
        assert(type(allies) == "table" and next(allies) == nil,
            "enabled empty input must not fabricate allies")
        "##,
    )
    .unwrap();
}

#[test]
fn recent_allies_publish_renamed_flags_and_complete_nested_rows() {
    let env = populated_env();
    env.exec(
        r##"
        assert(select("#", C_RecentAllies.GetRecentAllies()) == 1)
        local allies = C_RecentAllies.GetRecentAllies()
        assert(#allies == 2, "expected both explicit ally rows")
        local first, second = allies[1], allies[2]
        local s, other = first.stateData, second.stateData
        assert(s.friendRequestSentThisSession == true)
        assert(other.friendRequestSentThisSession == false)
        assert(s.hasFriendRequestPending == nil and other.hasFriendRequestPending == nil)
        assert(s.isOnline == true and s.isDND == false and s.isAFK == true)
        assert(s.isConvertedLegacyFriend == false)
        assert(s.pinExpirationDate == 1760100000 and s.currentLocation == "Dornogal")
        assert(other.isOnline == false and other.isDND == true and other.isAFK == false)
        assert(other.isConvertedLegacyFriend == true)
        assert(other.pinExpirationDate == nil and other.currentLocation == nil)
        local c = first.characterData
        assert(c.guid == "Player-100-00000001" and c.name == "Aster")
        assert(c.fullName == "Aster-FixtureRealm" and c.realmName == "FixtureRealm")
        assert(c.level == 80 and c.classID == 2 and c.raceID == 1 and c.sex == 3)
        assert(second.characterData.guid == "Player-100-00000002")
        assert(second.characterData.name == "Birch")
        assert(second.characterData.fullName == "Birch-FixtureRealm")
        local data = first.interactionData
        assert(data.note == "Helped with the dungeon" and #data.interactions == 2)
        local interaction = data.interactions[1]
        assert(interaction.type == 2 and interaction.timestamp == 1760000123)
        assert(interaction.description == "Completed fixture dungeon")
        local context = interaction.contextData
        assert(context.itemID == 210001 and context.locationName == "Fixture dungeon")
        assert(context.activityDifficultyID == 8 and context.activityDifficultyLevel == 12)
        local trade = data.interactions[2]
        assert(trade.type == 3 and trade.timestamp == 1760000456)
        assert(trade.description == "Traded supplies")
        assert(type(trade.contextData) == "table" and next(trade.contextData) == nil)
        assert(second.interactionData.note == nil)
        assert(type(second.interactionData.interactions) == "table")
        assert(next(second.interactionData.interactions) == nil)
        "##,
    )
    .unwrap();
}

#[test]
fn recent_allies_result_mutation_does_not_change_input_or_next_snapshot() {
    let env = populated_env();
    let original = env.state().borrow().recent_allies.clone();
    env.exec(
        r#"
        local first = C_RecentAllies.GetRecentAllies()
        local snapshot = C_RecentAllies.GetRecentAllies()
        assert(first ~= snapshot and first[1] ~= snapshot[1])
        first[1].stateData.friendRequestSentThisSession = false
        first[1].characterData.name = "Changed"
        first[1].interactionData.note = "Changed"
        first[1].interactionData.interactions[1].contextData.itemID = 999
        first[1].interactionData.interactions[2] = nil
        first[2] = nil
        for _, result in ipairs({snapshot, C_RecentAllies.GetRecentAllies()}) do
            assert(#result == 2)
            assert(result[1].stateData.friendRequestSentThisSession == true)
            assert(result[1].characterData.name == "Aster")
            assert(result[1].interactionData.note == "Helped with the dungeon")
            assert(#result[1].interactionData.interactions == 2)
            assert(result[1].interactionData.interactions[1].contextData.itemID == 210001)
            assert(result[2].stateData.friendRequestSentThisSession == false)
        end
        "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().recent_allies, original);
}
