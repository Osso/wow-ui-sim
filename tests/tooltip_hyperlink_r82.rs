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
    #[test]
    fn tooltip_hyperlink_r82_trade_returns_false() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(r#"
            -- R82 Q1: trade submits a request and returns false in the service.
            assert(C_TooltipInfo.GetHyperlink('trade:Player-1-00000001:2259:1') == false)
        "#).unwrap();
    }

}

#[test]
fn tooltip_hyperlink_r82_parses_item_request_fields_without_owned_identity() {
    use wow_ui_sim::c_api::c_tooltip_info_hyperlink::parse::{HyperlinkRequest, parse};
    // R82 Q2 item layout and separate wrapper fields; no semantic names inferred.
    let text = "item:25:11:12:13:14:15:16:17:50:2:0:3:2:5809:5810:2:9:70:10:80:2:101:102:1:103:0:Player-1-ABC:99";
    let HyperlinkRequest::Item(request) = parse(text, Some(21), Some(22), true, 7).unwrap() else {
        panic!("item request");
    };
    assert_eq!(request.link.descriptor.item_id, 25);
    assert_eq!(request.link.enchant_id, 11);
    assert_eq!(request.link.socket_item_ids, [12, 13, 14]);
    assert_eq!(request.link.fourth_gem_scalar, 15);
    assert_eq!(request.link.link_level, 50);
    assert_eq!(request.link.request_context, 2);
    assert_eq!(request.link.upgrade_mask, 0);
    let bonus = request.link.descriptor.bonus_context.unwrap();
    assert_eq!(bonus.creation_context, 3);
    assert_eq!(bonus.bonus_list_ids, [5809, 5810]);
    assert_eq!(
        request
            .link
            .descriptor
            .modifiers
            .iter()
            .map(|m| (m.key, m.value))
            .collect::<Vec<_>>(),
        [(9, 70), (10, 80)]
    );
    assert_eq!(
        request.link.socket_bonus_groups,
        [vec![101, 102], vec![103], vec![]]
    );
    assert_eq!(request.link.crafter_guid, "Player-1-ABC");
    assert_eq!(request.link.trailing_scalar, 99);
    assert_eq!(
        (
            request.optional_arg1,
            request.optional_arg2,
            request.hide_vendor_price
        ),
        (21, 22, true)
    );
    let HyperlinkRequest::Item(default) = parse(text, None, None, false, 7).unwrap() else {
        panic!("default item");
    };
    assert_eq!((default.optional_arg1, default.optional_arg2), (0, 7));
}

#[test]
fn tooltip_hyperlink_r82_spell_difficulty_is_not_rank() {
    use wow_ui_sim::c_api::c_tooltip_info_hyperlink::parse::{HyperlinkRequest, parse};
    // R82 Q2: mode 0, link, no ordinary overrides, difficulty -1 by default.
    for (text, difficulty) in [
        ("spell:19750", -1),
        ("|cff00ff00|Hspell:19750:23|h[spell]|h|r", 23),
    ] {
        let HyperlinkRequest::Spell(request) = parse(text, Some(31), Some(32), false, 9).unwrap()
        else {
            panic!("spell request");
        };
        assert_eq!(request.spell_id, 19750);
        assert_eq!(request.difficulty, difficulty);
        assert_eq!(request.mode, 0);
        assert!(request.is_link);
        assert!(!request.resolve_overrides);
        assert_eq!((request.optional_arg1, request.optional_arg2), (31, 32));
    }
}

#[test]
fn tooltip_hyperlink_r82_rank_inputs_and_marker_inventory() {
    use wow_ui_sim::c_api::c_tooltip_info_hyperlink::parse::{
        HyperlinkRequest, MARKERS, Marker, parse,
    };
    // R82 Q1: exactly 22 routes. Neither battle-pet spelling is in this chain.
    assert_eq!(MARKERS.len(), 22);
    assert!(parse("battlepet:1", None, None, false, 0).is_none());
    assert!(parse("battle-pet:1", None, None, false, 0).is_none());
    for (marker, tag) in MARKERS {
        let suffix = match marker {
            Marker::Item => "25:0:0:0:0:0:0:0:50:1:0:2:0:0:0:0:0::0",
            Marker::Trade => "Player-1-1:2:3",
            _ => "1:2:3",
        };
        assert!(
            parse(&format!("prefix {tag}{suffix}"), None, None, false, 0).is_some(),
            "{tag}"
        );
    }
    // R82 Q2: essence rank is unsigned, conduit rank is signed-clamped.
    assert_eq!(
        parse("azessence:1:-1", None, None, false, 0),
        Some(HyperlinkRequest::AzeriteEssence {
            id: 1,
            rank: u32::MAX
        })
    );
    assert_eq!(
        parse("conduit:1:-1", None, None, false, 0),
        Some(HyperlinkRequest::Conduit { id: 1, rank: 1 })
    );
    assert_eq!(
        parse("apower:41:258:259", None, None, false, 0),
        Some(HyperlinkRequest::ArtifactPower {
            id: 41,
            base: 2,
            total: 3
        })
    );
}
