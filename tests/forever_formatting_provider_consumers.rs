//! Original Forever Blizzard consumers use the cached formatting providers.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::{WowLuaEnv, state::QuestRewardCurrency};

#[test]
fn cached_custom_aura_container_merges_supplied_group_options() {
    let env = crate::common::aura_container_harness::load_env();
    env.exec(
        r#"
        local container = CreateFrame('ManagedAuraContainer', nil, UIParent, 'CustomAuraContainerTemplate')
        container:AddAuraGroup('format-provider', 'HELPFUL', {
            maxFrameCount = 2,
            initializeFrame = function(frame) frame:SetSize(27, 19) end,
        })
        local count = container:GetAuraGroupFrameCount('format-provider')
        assert(count == 2, 'original custom-aura consumer applied maxFrameCount')
        local frame = container:GetAuraGroupFrame('format-provider', 1)
        assert(frame:GetWidth() == 27 and frame:GetHeight() == 19,
            'original custom-aura consumer applied initializeFrame')
        "#,
    )
    .expect("cached CustomAuraContainer should merge and apply group options");
}

#[test]
fn loaded_adventure_map_displays_currency_with_original_color_provider() {
    let env = adventure_map_env();
    env.exec(
        r#"
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
            rewardPool = {
                ReleaseAll = function() end,
                GetNumActive = function() return 1 end,
            },
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
        "#,
    )
    .expect("loaded Adventure Map should display the original currency provider color");
}

fn adventure_map_env() -> WowLuaEnv {
    let env = crate::common::blizzard_addon_harness::with_blizzard_addon_smoke_shape(
        &[],
        &[],
        |env, _| env.clone(),
    );
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
