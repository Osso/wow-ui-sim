//! Behavioral fixtures for the state-backed renown reward producer.
#![cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]

use wow_ui_sim::c_api::c_major_factions::RenownRewardInfo;
use wow_ui_sim::lua_api::{MajorFactionData, RenownLevelInfo, WowLuaEnv};

fn register_faction_levels(env: &WowLuaEnv, faction_id: i64) {
    let state = env.state();
    let mut state = state.borrow_mut();
    state.major_factions.insert(
        faction_id,
        MajorFactionData {
            faction_id,
            name: format!("Fixture faction {faction_id}"),
            expansion_filter: 9,
            max_level: 8,
            renown_level: 7,
            renown_reputation_earned: 100,
            renown_level_threshold: 2500,
            ui_priority: 1,
            is_unlocked: true,
            unlock_description: None,
            celebration_sound_kit: 0,
            renown_fanfare_sound_kit_id: 0,
            texture_kit: "majorfactions_dreamwardens".into(),
            faction_font_color: (1.0, 1.0, 1.0),
            ..Default::default()
        },
    );
    state.major_faction_renown_levels.insert(
        faction_id,
        [7, 8]
            .into_iter()
            .map(|level| RenownLevelInfo {
                faction_id,
                level,
                locked: false,
                is_milestone: false,
                is_capstone: false,
            })
            .collect(),
    );
}

fn sparse_reward(renown_reward_id: i64, ui_order: i32) -> RenownRewardInfo {
    RenownRewardInfo {
        renown_reward_id,
        ui_order,
        is_account_unlock: false,
        item_id: None,
        spell_id: None,
        mount_id: None,
        transmog_id: None,
        transmog_set_id: None,
        title_mask_id: None,
        transmog_illusion_source_id: None,
        icon: None,
        name: None,
        description: None,
        toast_description: None,
        reward_type: None,
        is_collected: None,
    }
}

fn fixture_rewards() -> Vec<RenownRewardInfo> {
    vec![
        RenownRewardInfo {
            is_account_unlock: true,
            item_id: Some(210001),
            spell_id: Some(450001),
            mount_id: Some(1801),
            transmog_id: Some(190001),
            transmog_set_id: Some(3301),
            title_mask_id: Some(501),
            transmog_illusion_source_id: Some(901),
            icon: Some(123456),
            name: Some("Wardens reward".into()),
            description: Some("Fixture description".into()),
            toast_description: Some("Fixture toast".into()),
            reward_type: Some(2),
            is_collected: Some(true),
            ..sparse_reward(7101, 30)
        },
        RenownRewardInfo {
            item_id: Some(210002),
            name: Some("Uncollected reward".into()),
            is_collected: Some(false),
            ..sparse_reward(7102, 10)
        },
        sparse_reward(7103, 20),
    ]
}

fn reward_fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("renown reward environment");
    register_faction_levels(&env, 2507);
    env.state()
        .borrow_mut()
        .major_faction_renown_rewards
        .insert((2507, 7), fixture_rewards());
    env
}

#[test]
fn major_faction_renown_rewards_public_selectors_allow_tainted_callers() {
    let env = reward_fixture();
    env.exec(
        r#"
        local function addon()
            local rewards = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
            assert(#rewards == 3 and rewards[1].renownRewardID == 7101)
            assert(not issecure(), "query must not clear caller taint")
        end
        debug.setobjecttaint(addon, "RenownRewardsProbe")
        local ok, err = pcall(addon)
        assert(ok, err)
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_secret_selectors_require_secure_callers() {
    let env = reward_fixture();
    env.exec(
        r#"
        local secretFaction, secretLevel = secretwrap(2507), secretwrap(7)
        for _, args in ipairs({{secretFaction, 7}, {2507, secretLevel},
            {secretFaction, secretLevel}}) do
            local ok, rewards = pcall(C_MajorFactions.GetRenownRewardsForLevel,
                args[1], args[2])
            assert(ok and #rewards == 3 and rewards[1].renownRewardID == 7101,
                "secure caller must resolve each secret selector")
            local function addon()
                return C_MajorFactions.GetRenownRewardsForLevel(args[1], args[2])
            end
            debug.setobjecttaint(addon, "RenownRewardsProbe")
            assert(not pcall(addon), "tainted caller must reject each secret selector")
        end
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_require_numeric_selectors() {
    let env = reward_fixture();
    env.exec(
        r#"
        local query = C_MajorFactions.GetRenownRewardsForLevel
        assert(not pcall(query))
        assert(not pcall(query, 2507))
        assert(not pcall(query, nil, 7))
        assert(not pcall(query, "2507", 7))
        assert(not pcall(query, 2507, "7"))
        assert(not pcall(query, false, 7))
        assert(not pcall(query, 2507, {}))
        assert(next(query(2507.5, 7)) == nil, "do not truncate faction selector")
        assert(next(query(2507, 7.5)) == nil, "do not truncate level selector")
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_return_independent_snapshots() {
    let env = reward_fixture();
    env.exec(
        r#"
        local a = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        local b = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        assert(#a == 3 and #b == 3)
        assert(a ~= b and a[1] ~= b[1])
        a[1].name, a[1].isCollected, a[1].itemID = "Lua mutation", false, nil
        a[2] = nil
        assert(b[1].name == "Wardens reward" and b[1].isCollected == true)
        assert(b[1].itemID == 210001 and b[2].renownRewardID == 7102)
        local c = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        assert(#c == 3 and c[1].name == "Wardens reward")
        assert(c[1].isCollected == true and c[1].itemID == 210001)
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_default_has_no_fabricated_rows() {
    let env = WowLuaEnv::new().expect("renown reward environment");
    assert!(env.state().borrow().major_faction_renown_rewards.is_empty());
    register_faction_levels(&env, 2507);
    assert!(env.state().borrow().major_faction_renown_rewards.is_empty());
    env.exec(
        r#"
        local rewards = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        assert(type(rewards) == "table")
        assert(next(rewards) == nil, "no fabricated default rewards")
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_publish_identity_and_optional_fields() {
    let env = reward_fixture();
    super::patch_12_0_0_struct_shapes::assert_shape(
        &env,
        "MajorFactionsDocumentation.lua",
        "MajorFactionRenownRewardInfo",
        "return C_MajorFactions.GetRenownRewardsForLevel(2507, 7)[1]",
    );
    env.exec(
        r#"
        local rewards = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        assert(type(rewards) == "table")
        assert(#rewards == 3, "expected three modeled renown rewards")
        local a, b, c = rewards[1], rewards[2], rewards[3]
        assert(a.renownRewardID == 7101 and a.uiOrder == 30)
        assert(b.renownRewardID == 7102 and b.uiOrder == 10)
        assert(c.renownRewardID == 7103 and c.uiOrder == 20)
        assert(a.isAccountUnlock == true)
        assert(b.isAccountUnlock == false and c.isAccountUnlock == false)
        assert(a.itemID == 210001 and a.spellID == 450001)
        assert(a.mountID == 1801 and a.transmogID == 190001)
        assert(a.transmogSetID == 3301 and a.titleMaskID == 501)
        assert(a.transmogIllusionSourceID == 901 and a.icon == 123456)
        assert(a.name == "Wardens reward")
        assert(a.description == "Fixture description")
        assert(a.toastDescription == "Fixture toast" and a.rewardType == 2)
        assert(a.isCollected == true and b.isCollected == false)
        assert(c.isCollected == nil)
        assert(b.itemID == 210002 and b.name == "Uncollected reward")
        for _, key in ipairs({"spellID", "mountID", "transmogID", "transmogSetID",
            "titleMaskID", "transmogIllusionSourceID", "icon", "description",
            "toastDescription", "rewardType"}) do
            assert(b[key] == nil, "second row omitted " .. key)
        end
        for _, key in ipairs({"itemID", "spellID", "mountID", "transmogID",
            "transmogSetID", "titleMaskID", "transmogIllusionSourceID", "icon",
            "name", "description", "toastDescription", "rewardType", "isCollected"}) do
            assert(c[key] == nil, "third row omitted " .. key)
        end
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_do_not_leak_across_pair_keys() {
    let env = reward_fixture();
    register_faction_levels(&env, 2508);
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state
            .major_faction_renown_rewards
            .insert((2507, 8), vec![sparse_reward(8101, 1)]);
        state
            .major_faction_renown_rewards
            .insert((2508, 7), vec![sparse_reward(7201, 2)]);
    }
    env.exec(
        r#"
        local a = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        local b = C_MajorFactions.GetRenownRewardsForLevel(2507, 8)
        local c = C_MajorFactions.GetRenownRewardsForLevel(2508, 7)
        assert(#a == 3 and #b == 1 and #c == 1, "pair-specific reward counts")
        assert(a[1].renownRewardID == 7101)
        assert(b[1].renownRewardID == 8101 and b[1].uiOrder == 1)
        assert(c[1].renownRewardID == 7201 and c[1].uiOrder == 2)
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_missing_registered_pair_is_empty() {
    let env = reward_fixture();
    env.exec(
        r#"
        assert(C_MajorFactions.GetMajorFactionData(2507).factionID == 2507)
        assert(C_MajorFactions.GetRenownLevels(2507)[2].level == 8)
        local rewards = C_MajorFactions.GetRenownRewardsForLevel(2507, 8)
        assert(type(rewards) == "table" and next(rewards) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn major_faction_renown_rewards_model_updates_reach_next_query() {
    let env = reward_fixture();
    env.exec(
        r#"
        local rewards = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        assert(#rewards == 3, "initial modeled reward count")
        assert(rewards[1].isCollected == true)
        "#,
    )
    .unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let rewards = state
            .major_faction_renown_rewards
            .get_mut(&(2507, 7))
            .unwrap();
        rewards[0].is_collected = Some(false);
        rewards[0].name = Some("Updated reward".into());
        rewards.pop();
    }
    env.exec(
        r#"
        local rewards = C_MajorFactions.GetRenownRewardsForLevel(2507, 7)
        assert(#rewards == 2, "updated modeled reward count")
        assert(rewards[1].renownRewardID == 7101)
        assert(rewards[1].isCollected == false and rewards[1].name == "Updated reward")
        assert(rewards[2].renownRewardID == 7102)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .major_faction_renown_rewards
        .remove(&(2507, 7));
    env.exec("assert(next(C_MajorFactions.GetRenownRewardsForLevel(2507, 7)) == nil)")
        .unwrap();
}
