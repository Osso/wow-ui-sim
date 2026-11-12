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

    #[test]
    fn tooltip_hyperlink_r82_apower_rank_colour_uses_adjusted_bonus() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.state().borrow_mut().hyperlink_tooltip.artifact_display.insert(41,
            wow_ui_sim::c_api::c_tooltip_info_hyperlink::inputs::ArtifactDisplay {
                normal_maximum: 3, rank_limit: 6, use_normal_maximum: false,
            });
        env.exec(r#"
            -- R82 Q3 apower: rank line type 0, followed by type-1 blank.
            -- Fixture row 41 is INFERRED 70391 data. No assertion of catalog text or catalog numeric values.
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            for _, case in ipairs({{'apower:41:1:1', HIGHLIGHT_FONT_COLOR, 1, 3}, {'apower:41:2:1', INCREASE_STAT_COLOR, 1, 2}, {'apower:41:1:0', INCREASE_STAT_COLOR, 0, 2}, {'apower:41:1:2', INCREASE_STAT_COLOR, 2, 4}}) do
                GameTooltip:SetHyperlink(case[1])
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(data and data.lines and data.lines[2], 'apower prefix missing')
                assert(data.lines[1].type == 0 and data.lines[2].type == 0 and data.lines[3].type == 1)
                assert(data.lines[2].leftText == TOOLTIP_TALENT_RANK:format(case[3],case[4]), 'R82b rank operands')
                local colour = data.lines[2].leftColor
                assert(math.abs(colour.r-case[2].r)<0.005 and math.abs(colour.g-case[2].g)<0.005 and math.abs(colour.b-case[2].b)<0.005, 'bonus colour')
            end
        "#).unwrap();
    }

    #[test]
    fn tooltip_hyperlink_r82_conduit_clamps_rank_and_suppresses_soulbound() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(r#"
            -- R82 Q2/Q3 mode 1, signed clamp, descriptor prefix, no soulbound row.
            -- Fixture conduit 5 is INFERRED 70391 data, not native 70688 proof.
            for _, rank in ipairs({0,1,-1}) do
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetHyperlink('conduit:5:' .. rank)
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(data and data.lines and #data.lines >= 3, 'conduit prefix missing')
                assert(data.lines[1].type == 0)
                local hasLevel = false
                for _, line in ipairs(data.lines) do
                    assert(line.leftText ~= ITEM_SOULBOUND, 'link mode must suppress soulbound')
                    if line.type == 31 then hasLevel = type(line.itemLevel) == 'number' end
                end
                assert(hasLevel, 'conduit item-level metadata missing')
            end
        "#).unwrap();
    }

    #[test]
    fn tooltip_hyperlink_r82_essence_uses_supplied_rank_spell_selection() {
        use wow_ui_sim::c_api::c_tooltip_info_hyperlink::inputs::EssenceSpells;
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.state().borrow_mut().hyperlink_tooltip.essence_spells.insert((7,1), EssenceSpells {
            main_spell_id: 19750, passive_spell_id: 53563,
            main_heading: "Supplied main power".into(), passive_heading: "Supplied passive power".into(),
        });
        env.exec(r#"
            -- R82 Q2/Q3 mode 2: identity and main/passive ordering, no source/next rank.
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetHyperlink('azessence:7:0')
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.type == 8 and data.id == 7)
            assert(data.lines[1].leftText == 'Supplied main power')
            assert(data.lines[2].leftText == C_Spell.GetSpellName(19750))
            local passive
            for i,line in ipairs(data.lines) do
                if line.leftText == 'Supplied passive power' then passive = i end
            end
            assert(passive and passive > 2)
            assert(data.lines[passive+1].leftText == C_Spell.GetSpellName(53563))
        "#).unwrap();
    }

    #[test]
    fn tooltip_hyperlink_r82_quest_only_publishes_supplied_title() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        let id = {
            let mut state = env.state().borrow_mut();
            let quest = state.quest_log_entries.entries.first_mut().expect("seeded quest");
            quest.title = "Supplied hyperlink quest".into();
            quest.quest_id
        };
        env.exec(&format!(r#"
            -- R82 Q3 quest prefix; objective/state helpers are UNKNOWN.
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetHyperlink('quest:{id}')
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(data and data.type == 23 and data.id == {id})
            assert(#data.lines == 1 and data.lines[1].type == 0)
            assert(data.lines[1].leftText == 'Supplied hyperlink quest')
        "#)).unwrap();
    }

    #[cfg(feature = "client-retail")]
    #[test]
    fn tooltip_hyperlink_r82_pvptal_uses_primary_spell_without_annotation() {
        use wow_ui_sim::c_api::c_tooltip_info_pvp_talent::PvpTalentTooltipRecord;
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.state().borrow_mut().pvp_talent_tooltip.catalog.insert(9001,PvpTalentTooltipRecord {
            spell_id:19750, overrides_spell_id:1459, level_required:99,
            grant_rejected_by_item:true, ..Default::default()
        });
        env.exec(r#"
            -- R82 Q2/Q3: primary row spell, secondaryFlag=true suppresses annotation.
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetHyperlink('pvptal:9001')
            local data = GameTooltip:GetPrimaryTooltipData()
            local spell = C_TooltipInfo.GetHyperlink('spell:19750')
            assert(data and data.id == 19750 and #data.lines == #spell.lines)
            for i,line in ipairs(data.lines) do
                assert(line.leftText == spell.lines[i].leftText)
            end
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

#[test]
fn tooltip_hyperlink_r82b_artifact_display_math_uses_explicit_context() {
    use wow_ui_sim::c_api::c_tooltip_info_hyperlink::{
        inputs::ArtifactDisplay, parse::artifact_ranks,
    };
    let mut display = ArtifactDisplay {
        normal_maximum: 3,
        rank_limit: 6,
        use_normal_maximum: false,
    };
    // R82b: zero displays zero but looks up rank 1. Max adjusts by total-base.
    let ranks = artifact_ranks(0, 0, 0, &display);
    assert_eq!(
        (
            ranks.displayed_total,
            ranks.lookup_rank,
            ranks.displayed_maximum
        ),
        (0, 1, 3)
    );
    let ranks = artifact_ranks(0, 1, 3, &display);
    assert_eq!(
        (
            ranks.displayed_total,
            ranks.displayed_maximum,
            ranks.adjusted_bonus
        ),
        (3, 5, 2)
    );
    assert_eq!(artifact_ranks(0, 0, 8, &display).displayed_maximum, 6);
    assert_eq!(artifact_ranks(64, 0, 8, &display).displayed_total, 1);
    assert_eq!(artifact_ranks(16, 1, 2, &display).adjusted_bonus, 0);
    assert_eq!(artifact_ranks(16, 2, 1, &display).adjusted_bonus, u32::MAX);
    display.use_normal_maximum = true;
    assert_eq!(artifact_ranks(0, 1, 3, &display).displayed_maximum, 3);
}
