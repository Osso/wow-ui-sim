//! WM94 §5: the original `SetConduit` accessor composes the conduit heading
//! block, the item-level and soulbound rows, and the conduit spell text.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

tooltip_selection_getters_fixture! {
    #[test]
    fn tooltip_conduit_original_accessor_reports_rank_heading_and_spell_text() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            for _, case in ipairs({
                -- conduit, rank, spell name, type label, item level, quality
                {5, 1, 'Stalwart Guardian', CONDUIT_TYPE_POTENCY, 145, 2},
                {5, 5, 'Stalwart Guardian', CONDUIT_TYPE_POTENCY, 200, 4},
                {8, 3, 'Inspiring Presence', CONDUIT_TYPE_FINESSE, 171, 3},
                {11, 4, 'Ashen Juggernaut', CONDUIT_TYPE_ENDURANCE, 184, 4},
            }) do
                local conduit_id, rank, name, label, item_level, quality =
                    case[1], case[2], case[3], case[4], case[5], case[6]
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetConduit(conduit_id, rank)
                local info = GameTooltip:GetPrimaryTooltipInfo()
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(info and info.getterName == 'GetConduit', 'conduit accessor missing')
                assert(data and data.type == Enum.TooltipDataType.EnhancedConduit,
                    'conduit tooltip type')
                local lines = data.lines
                local name_line = lines[1]
                assert(name_line.leftText == name, 'quality-coloured spell name')
                assert(name_line.type == 13, 'spell-name line type')
                local expected = ({[2] = {0.12, 1.0, 0.0}, [3] = {0.0, 0.44, 0.87}, [4] = {0.64, 0.21, 0.93}})[quality]
                -- The published colour rounds each channel to a byte (D56).
                local tolerance = 1 / 255
                assert(math.abs(name_line.leftColor.r - expected[1]) <= tolerance
                    and math.abs(name_line.leftColor.g - expected[2]) <= tolerance
                    and math.abs(name_line.leftColor.b - expected[3]) <= tolerance,
                    'name colour must follow the rank quality')
                assert(lines[2].leftText == label, 'conduit type label')
                assert(lines[2].type == 0, 'label is a plain line')
                assert(lines[3].type == 31, 'item-level line type')
                assert(lines[3].itemLevel == item_level, 'item-level numeric field')
                assert(lines[3].leftText == string.format(ITEM_LEVEL, item_level))
                assert(lines[4].type == 20 and lines[4].leftText == ITEM_SOULBOUND,
                    'soulbound row')
                assert(lines[5].type == 34 and #lines[5].leftText > 0,
                    'conduit spell description text')
                assert(#lines == 5, 'no extra conduit rows')
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_conduit_clamps_rank_below_one_and_rejects_missing_rows() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            local function snapshot(rank)
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetConduit(5, rank)
                local data = GameTooltip:GetPrimaryTooltipData()
                local parts = {}
                for index, line in ipairs(data.lines) do
                    parts[index] = tostring(line.leftText)
                end
                return table.concat(parts, '|')
            end
            assert(snapshot(1) == snapshot(0), 'rank below one clamps to rank one')
            assert(select('#', C_TooltipInfo.GetConduit(5, 16)) == 0,
                'a rank past the supplied rows must not select a lower rank')
            assert(select('#', C_TooltipInfo.GetConduit(999999, 1)) == 0,
                'an unknown conduit must take the missing route')
            "#,
        )
        .unwrap();
    }
}
