//! WM108 §B: the original `SetEnhancedConduit` accessor uses the conduit id for
//! membership, adds two ranks up to the row count, and keeps quality/level on
//! the original clamped rank.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

tooltip_selection_getters_fixture! {
    #[test]
    fn tooltip_enhanced_conduit_original_accessor_adjusts_only_the_spell_rank() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            for _, case in ipairs({
                -- conduit, rank, item level, quality
                {5, 1, 145, 2},
                {5, 13, 304, 4},
                {5, 14, 317, 4},
                {8, 3, 171, 3},
            }) do
                local conduit_id, rank, item_level, quality =
                    case[1], case[2], case[3], case[4]
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetEnhancedConduit(conduit_id, rank)
                local info = GameTooltip:GetPrimaryTooltipInfo()
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(info and info.getterName == 'GetEnhancedConduit',
                    'enhanced-conduit accessor missing')
                assert(data and data.type == Enum.TooltipDataType.EnhancedConduit)
                local lines = data.lines
                local function dump()
                    local parts = {}
                    for index, line in ipairs(lines) do
                        parts[index] = ('%d:%s|%s'):format(index, tostring(line.type),
                            tostring(line.leftText))
                    end
                    return table.concat(parts, ' ')
                end
                assert(#lines == 6, 'enhanced conduit row count: ' .. dump())
                assert(lines[1].type == 13, 'spell-name line type: ' .. dump())
                local expected = ({[2] = {0.12, 1.0, 0.0}, [3] = {0.0, 0.44, 0.87}, [4] = {0.64, 0.21, 0.93}})[quality]
                local tolerance = 1 / 255
                assert(math.abs(lines[1].leftColor.r - expected[1]) <= tolerance
                    and math.abs(lines[1].leftColor.g - expected[2]) <= tolerance
                    and math.abs(lines[1].leftColor.b - expected[3]) <= tolerance,
                    'name colour follows the original rank quality: ' .. dump())
                assert(lines[2].type == 0 and lines[2].leftText ~= '', 'conduit type label')
                assert(lines[3].type == 0 and lines[3].leftText == SOULBIND_CONDUIT_ENHANCED,
                    'enhanced label: ' .. dump())
                assert(lines[4].type == 31 and lines[4].itemLevel == item_level,
                    'item level uses the original rank: ' .. dump())
                assert(lines[5].type == 20 and lines[5].leftText == ITEM_SOULBOUND)
                assert(lines[6].type == 34 and #lines[6].leftText > 0,
                    'spell description text')
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_enhanced_conduit_rejects_unknown_conduits() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            assert(select('#', C_TooltipInfo.GetEnhancedConduit(999999, 1)) == 0,
                'an unknown conduit must take the missing route')
            "#,
        )
        .unwrap();
    }
}
