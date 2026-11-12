//! R82 Q1/Q2/Q3 (70688): substring dispatch, quantity and bounded unknowns.
#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

tooltip_selection_getters_fixture! {
    #[test]
    fn tooltip_hyperlink_r82_raw_and_coloured_currency_preserve_quantity() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(r#"
            -- R82 Q1 marker search; Q2 currency quantity defaults to zero, no /100.
            for _, link in ipairs({'currency:1792:1234', '|cff00ff00|Hcurrency:1792:1234|h[Points]|h|r', 'prefix currency:1792:1234'}) do
                local data = C_TooltipInfo.GetHyperlink(link)
                assert(data and data.id == 1792 and data.quantity == 1234, 'currency request lost')
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetHyperlink(link)
                local shown = GameTooltip:GetPrimaryTooltipData()
                assert(shown and shown.id == 1792 and shown.quantity == 1234, 'accessor request lost')
            end
            assert(C_TooltipInfo.GetHyperlink('currency:1792').quantity == 0)
            assert(C_TooltipInfo.GetHyperlink('currency:0x700:0x4D2').quantity == 1234)
        "#).unwrap();
    }

    #[test]
    fn tooltip_hyperlink_r82_marker_priority_precedes_text_position() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(r#"
            -- R82 Q1: item is searched before currency, independent of location.
            local data = C_TooltipInfo.GetHyperlink('currency:1792:20 text item:25:0:0:0:0:0:0:0:50:1:0:2:0:0:0:0:0::0')
            assert(data and data.type == Enum.TooltipDataType.Item, 'item route must win')
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetHyperlink('prefix spell:19750')
            local shown = GameTooltip:GetPrimaryTooltipData()
            assert(shown and shown.id == 19750, 'raw substring spell route missing')
        "#).unwrap();
    }

    #[test]
    fn tooltip_hyperlink_r82_no_producer_markers_do_not_fall_through() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(r#"
            -- R82 Q1: these four markers call no producer; battlepet is not a branch.
            -- Check no fabricated populated data, not UNKNOWN native Lua miss arity.
            for _, tag in ipairs({'urlIndex', 'storecategory', 'perksactivity', 'initiativetask', 'battlepet', 'battle-pet'}) do
                local data = C_TooltipInfo.GetHyperlink(tag .. ':1')
                assert(not (type(data) == 'table' and data.lines and #data.lines > 0), tag .. ' invented rows')
            end
            -- R82 Q2/Q3: unit object lookup is UNKNOWN, not a player-name guess.
            local data = C_TooltipInfo.GetHyperlink('unit:Player-1-00000001')
            assert(not (type(data) == 'table' and data.lines and #data.lines > 0), 'unit rows are unmodelled')
        "#).unwrap();
    }
}
