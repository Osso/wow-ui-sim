//! Original Forever Blizzard consumers use the cached formatting providers.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::{WowLuaEnv, state::{AuraInfo, QuestRewardCurrency}};

const ADVENTURE_MAP_CURRENCY_PROBE: &str = r#"
    local loaded, reason = C_AddOns.LoadAddOn('Blizzard_AdventureMap')
    assert(loaded, 'Blizzard_AdventureMap must load: ' .. tostring(reason))
    local provider = GetColorForCurrencyReward
    local expected = { provider(1166, 5):GetRGB() }
    local observed
    GetColorForCurrencyReward = function(currencyID, quantity)
        observed = { currencyID, quantity }
        return provider(currencyID, quantity)
    end
    local dialog = {
        questID = 80000,
        rewardPool = { ReleaseAll = function() end, GetNumActive = function() return 1 end },
        Rewards = {
            Show = function() end,
            SetHeight = function() end,
            SetTexCoord = function() end,
        },
        RewardsHeader = { Show = function() end },
        AddReward = function()
            return { Count = { SetTextColor = function(_, r, g, b) displayed = {r, g, b} end } }
        end,
    }
    AdventureMapQuestChoiceDialogMixin.RefreshRewards(dialog)
    assert(observed[1] == 1166 and observed[2] == 5,
        'original Adventure Map reward consumer called the currency color provider')
    assert(displayed[1] == expected[1] and displayed[2] == expected[2] and displayed[3] == expected[3],
        'original Adventure Map reward consumer displayed the provider RGB')
"#;

#[test]
fn cached_custom_aura_container_merges_supplied_group_options() {
    let env = custom_aura_container_env();
    env.exec(
        r#"
        container = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        container:SetUnit('player')
        container:AddAuraGroup('format-provider', 'HELPFUL', {
            maxFrameCount = 2,
            initializeFrame = function(frame) frame:SetSize(27, 19) end,
        })
        "#,
    )
    .unwrap();
    env.fire_unit_aura_full_update("player").unwrap();
    env.fire_on_update(0.016).unwrap();
    env.exec(
        r#"
        local count = container:GetAuraGroupFrameCount('format-provider')
        assert(count > 0, 'original custom-aura consumer created option-configured frames')
        assert(count <= 2, 'supplied maxFrameCount overwrites the original unbounded default')
        local frame = container:GetAuraGroupFrame('format-provider', 1)
        assert(frame:GetWidth() == 27 and frame:GetHeight() == 19,
            'original custom-aura consumer applied initializeFrame')
        "#,
    )
    .expect("cached CustomAuraContainer should merge and apply group options");
}

fn custom_aura_container_env() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    seed_three_auras(&env);
    env
}

fn seed_three_auras(env: &WowLuaEnv) {
    env.state().borrow_mut().player.buffs = (1..=3)
        .map(|id| AuraInfo {
            name: format!("Audit {id}"),
            spell_id: id,
            icon: id,
            duration: 30.0,
            expiration_time: 30.0,
            applications: 1,
            source_unit: "player".into(),
            is_helpful: true,
            is_raid: true,
            is_nameplate_only: false,
            is_stealable: false,
            can_apply_aura: true,
            is_from_player_or_player_pet: true,
            dispel_type: None,
            aura_instance_id: id,
        })
        .collect();
}

#[test]
fn cached_professions_filter_uses_original_tfilter() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_ProfessionsTemplates"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_ProfessionsTemplates"));
    env.exec(
        r#"
        local input = {{ itemID = 123 }, { currencyID = 456 }}
        local filtered = Professions.FilterReagentsByCurrencyID(input)
        assert(filtered ~= input, 'cached tFilter returns a new result table')
        assert(#filtered == 1 and filtered[1].currencyID == 456,
            'original Professions consumer keeps currency reagents')
        assert(#input == 2 and input[1].itemID == 123 and input[2].currencyID == 456,
            'original tFilter leaves the input table unchanged')
        "#,
    )
    .expect("cached Professions consumer should filter through the original tFilter");
}

#[test]
fn loaded_adventure_map_displays_currency_with_original_color_provider() {
    adventure_map_env()
        .exec(ADVENTURE_MAP_CURRENCY_PROBE)
        .expect("loaded Adventure Map should display the original currency provider color");
}

#[test]
fn cached_sharedxml_money_formatter_uses_texture_markup() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_SharedXML"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_SharedXML"));
    let formatted: String = env
        .eval(
            r#"
            local expected = GOLD_AMOUNT_TEXTURE_STRING:format(FormatLargeNumber(123), 0, 0)
                .. ' ' .. SILVER_AMOUNT_TEXTURE:format(45, 0, 0)
                .. ' ' .. COPPER_AMOUNT_TEXTURE:format(67, 0, 0)
            assert(GetMoneyString(1234567, true) == expected,
                'cached FormattingUtil must compose gold, silver, and copper texture strings')
            return GetMoneyString(1234567, true)
            "#,
        )
        .expect("cached FormattingUtil money formatter should return a string");
    assert!(
        formatted.contains("|T"),
        "expected coin texture markup: {formatted}"
    );
}

fn adventure_map_env() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    env.state().borrow_mut().selected_quest_log_id = Some(80000);
    env.state()
        .borrow_mut()
        .quest_log_entries
        .entries
        .iter_mut()
        .find(|entry| entry.quest_id == 80000)
        .expect("seed quest exists")
        .currency_rewards
        .push(QuestRewardCurrency {
            currency_id: 1166,
            name: "Timewarped Badge".into(),
            texture: "Interface\\Icons\\INV_Misc_Coin_01".into(),
            total_reward_amount: 5,
            base_reward_amount: Some(5),
        });
    env
}
