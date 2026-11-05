//! WM108 §A: the original `SetArtifactPowerByID` accessor selects one rank row,
//! forces rank 1 for `Flags & 0x40`, and suppresses the rank row for that bit
//! and for the end-game bit.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::lua_api::sim_substates::viewed_artifact::ArtifactPowerInfo;
use wow_ui_sim::lua_api::state::ArtifactInfo;

tooltip_selection_getters_fixture! {
    fn viewed_artifact() -> ArtifactInfo {
        ArtifactInfo {
            item_id: 128_000,
            alt_item_id: 0,
            name: "Fist of Ra-den".to_string(),
            icon: String::new(),
            total_xp: 0,
            points_spent: 0,
            quality: 6,
            artifact_appearance_id: 0,
            appearance_mod_id: 0,
            item_appearance_id: 0,
            alt_item_appearance_id: 0,
            alt_on_top: false,
            tier: 1,
            maxed: false,
            disabled: false,
            category: 0,
        }
    }

    fn power(current_rank: i32, bonus_ranks: i32, max_rank: i32) -> ArtifactPowerInfo {
        ArtifactPowerInfo {
            spell_id: 0,
            cost: 0,
            current_rank,
            max_rank,
            bonus_ranks,
            num_max_rank_bonus_from_tier: 0,
            prereqs_met: true,
            is_start: false,
            is_gold_medal: false,
            is_final: false,
            tier: 0,
            position: (0.0, 0.0),
            offset: None,
            linear_index: None,
        }
    }

    #[test]
    fn tooltip_artifact_power_orders_the_rank_row_and_spell_text() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        {
            let mut state = env.state().borrow_mut();
            state.viewed_artifact.info = Some(viewed_artifact());
            // Power 41: Flags 32, MaxPurchasableRank 3, Tier 0, spell "Highlord's Judgment".
            // The displayed maximum adds the artifact tier minus the power tier.
            state.viewed_artifact.powers.insert(41, power(2, 0, 3));
            // Power 42: same power shape with a bonus, which changes the rank colour only.
            state.viewed_artifact.powers.insert(42, power(1, 2, 3));
        }
        env.exec(
            r#"
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetArtifactPowerByID(41)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetArtifactPowerByID', 'accessor missing')
            assert(data and data.lines and #data.lines == 2, 'artifact-power data missing')
            local function dump()
                local parts = {}
                for index, line in ipairs(data.lines) do
                    parts[index] = ('%d:%d|%s'):format(index, line.type, tostring(line.leftText))
                end
                return table.concat(parts, ' ')
            end
            assert(#data.lines == 4, 'rank-row composition: ' .. dump())
            assert(data.lines[1].type == 13 and data.lines[1].leftText == "Highlord's Judgment")
            assert(data.lines[2].type == 0
                and data.lines[2].leftText == TOOLTIP_TALENT_RANK:format(2, 4),
                'rank row: ' .. dump())
            assert(data.lines[3].type == 1 and data.lines[3].leftText == ' ')
            assert(data.lines[4].type == 34 and #data.lines[4].leftText > 0)

            GameTooltip:SetArtifactPowerByID(42)
            local bonus = GameTooltip:GetPrimaryTooltipData()
            assert(bonus and bonus.lines[2].leftText == TOOLTIP_TALENT_RANK:format(1, 4),
                'bonus rank row: ' .. bonus.lines[2].leftText)
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_artifact_power_flag_64_forces_rank_one_and_hides_the_rank_row() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        {
            let mut state = env.state().borrow_mut();
            state.viewed_artifact.info = Some(viewed_artifact());
            // Power 1739: Flags 64, MaxPurchasableRank 3, spell "Netherlight Fortification".
            state.viewed_artifact.powers.insert(1739, power(3, 0, 3));
        }
        env.exec(
            r#"
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetArtifactPowerByID(1739)
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.lines and #data.lines == 2, 'artifact-power data missing')
            local function dump()
                local parts = {}
                for index, line in ipairs(data.lines) do
                    parts[index] = ('%d:%d|%s'):format(index, line.type, tostring(line.leftText))
                end
                return table.concat(parts, ' ')
            end
            assert(#data.lines == 2, 'flag-64 composition: ' .. dump())
            assert(data.lines[1].type == 13 and data.lines[1].leftText == 'Netherlight Fortification')
            assert(data.lines[2].type == 34, 'spell text without a rank row: ' .. dump())
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_artifact_power_unknown_power_is_a_miss() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            assert(select('#', C_TooltipInfo.GetArtifactPowerByID(999999)) == 0,
                'an unknown power must take the missing route')
            "#,
        )
        .unwrap();
    }
}
