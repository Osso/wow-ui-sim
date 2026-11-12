//! R81 Retail 70688: VERIFIED outer composition; host inputs stand in for UNKNOWN helpers.
use wow_ui_sim::c_api::spell_tooltip::SpellTooltipInput;
use wow_ui_sim::lua_api::WowLuaEnv;

fn seed(env: &WowLuaEnv, id: u32, input: SpellTooltipInput) {
    env.state()
        .borrow_mut()
        .spell_tooltip_inputs
        .insert(id, input);
}

#[test]
fn r81_spell_override_and_hyperlink_keep_distinct_identity_and_cost_range() {
    let env = WowLuaEnv::new().unwrap();
    seed(
        &env,
        19750,
        SpellTooltipInput {
            resolved_spell_id: Some(642),
            cost: Some("source cost".into()),
            ranges: vec!["source range".into()],
            ..Default::default()
        },
    );
    seed(
        &env,
        642,
        SpellTooltipInput {
            cost: Some("effective cost".into()),
            ranges: vec!["effective range".into()],
            ..Default::default()
        },
    );
    // R81 Q1: direct getter resolves overrides; a link keeps its parsed spell, not suppressing cost/range.
    env.exec(r#"
        local direct = C_TooltipInfo.GetSpellByID(19750)
        local link = C_TooltipInfo.GetHyperlink('spell:19750')
        assert(direct.id == 642 and link.id == 19750, 'R81 routing')
        assert(direct.lines[2].leftText == 'effective cost' and direct.lines[2].rightText == 'effective range')
        assert(link.lines[2].leftText == 'source cost' and link.lines[2].rightText == 'source range')
        GameTooltip:SetSpellByID(19750)
        assert(GameTooltip:GetSpell() == C_Spell.GetSpellName(642))
        assert(GameTooltipTextLeft2:GetText() == 'effective cost')
        GameTooltip:SetHyperlink('spell:19750')
        assert(GameTooltip:GetSpell() == C_Spell.GetSpellName(19750))
        assert(GameTooltipTextLeft2:GetText() == 'source cost')
        assert(GameTooltipTextRight2:GetText() == 'source range')
    "#).unwrap();
}

#[test]
fn r81_spell_details_follow_verified_order_and_side_publication() {
    let env = WowLuaEnv::new().unwrap();
    seed(
        &env,
        19750,
        SpellTooltipInput {
            overrides_spell_id: 642,
            azerite_annotation: Some("host essence".into()),
            cost: Some("host cost".into()),
            ranges: vec!["near".into(), "far".into()],
            cast_ms: Some(1500),
            recast_text: Some("host recast".into()),
            ..Default::default()
        },
    );
    // R81 Q1 ordinary-path bullets: replacement, Azerite, highlighted cost/ranges, cast/recast.
    env.exec(r#"
        local t = C_TooltipInfo.GetSpellByID(19750)
        assert(t.lines[2].leftText == format(REPLACES_SPELL, C_Spell.GetSpellName(642)), 'R81 replacement')
        assert(t.lines[2].type == 0 and t.lines[2].rightText == nil and t.lines[2].wrapText == nil)
        local r,g,b = t.lines[2].leftColor:GetRGB()
        local nr,ng,nb = NORMAL_FONT_COLOR:GetRGB()
        assert(math.abs(r-nr)<0.005 and math.abs(g-ng)<0.005 and math.abs(b-nb)<0.005)
        assert(t.lines[3].leftText == 'host essence' and t.lines[3].rightText == nil)
        assert(t.lines[4].leftText == 'host cost' and t.lines[4].rightText == 'near')
        assert(t.lines[5].leftText == 'far' and t.lines[5].rightText == nil)
        assert(t.lines[6].leftText == format(SPELL_CAST_TIME_SEC, 1.5))
        assert(t.lines[7].leftText == 'host recast')
        for i=4,7 do
            assert(t.lines[i].type == 0)
            local r,g,b = t.lines[i].leftColor:GetRGB()
            assert(r > 0.99 and g > 0.99 and b > 0.99)
        end
        GameTooltip:SetSpellByID(19750)
        assert(GameTooltipTextLeft4:GetText() == 'host cost' and GameTooltipTextRight4:GetText() == 'near')
    "#).unwrap();
    seed(
        &env,
        19750,
        SpellTooltipInput {
            cost: Some(String::new()),
            ranges: vec![String::new(), "range only".into()],
            ..Default::default()
        },
    );
    // R81 Q2 zero/empty bullet: zero override has no replacement; empty buffers add no text.
    env.exec(
        r#"
        local t = C_TooltipInfo.GetSpellByID(19750)
        assert(t.lines[2].leftText == 'range only' and t.lines[2].rightText == nil)
        assert(#t.lines == 3)
    "#,
    )
    .unwrap();
}

#[test]
fn r81_spell_cast_uses_localized_threshold_and_explicit_alternate() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q2 cast-time bullet: SEC below 60000; MIN at/above. Not a live cast formula.
    for (ms, token, value) in [
        (59999, "SPELL_CAST_TIME_SEC", 59.999),
        (60000, "SPELL_CAST_TIME_MIN", 1.0),
        (120000, "SPELL_CAST_TIME_MIN", 2.0),
    ] {
        seed(
            &env,
            19750,
            SpellTooltipInput {
                cast_ms: Some(ms),
                ..Default::default()
            },
        );
        env.exec(&format!("local t=C_TooltipInfo.GetSpellByID(19750); assert(t.lines[2].leftText == format({token}, {value}), 'R81 cast threshold'); GameTooltip:SetSpellByID(19750); assert(GameTooltipTextLeft2:GetText()==t.lines[2].leftText)")).unwrap();
    }
    // R81 Q3: sign alone does not establish Instant or next swing.
    for ms in [None, Some(0), Some(-1)] {
        seed(
            &env,
            19750,
            SpellTooltipInput {
                cast_ms: ms,
                ..Default::default()
            },
        );
        env.exec("assert(#C_TooltipInfo.GetSpellByID(19750).lines == 2, 'no guessed alternate')")
            .unwrap();
        seed(
            &env,
            19750,
            SpellTooltipInput {
                cast_ms: ms,
                alternate_cast_text: Some("host alternate".into()),
                ..Default::default()
            },
        );
        env.exec("assert(C_TooltipInfo.GetSpellByID(19750).lines[2].leftText == 'host alternate')")
            .unwrap();
    }
}

#[test]
fn r81_spell_range_and_cast_skip_masks() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q3 range/cast skip bullets, supplied as native attribute groups, not inferred DB2 mappings.
    for (mode, a0, a3, a13, effect, header, range, cast) in [
        (0, 0, 0, 0, 0, false, true, true),
        (1, 0, 0, 0, 0, false, false, false),
        (0, 4, 0, 0, 0, false, false, true),
        (0, 1024, 0, 0, 0, false, false, true),
        (0, 0, 1073741824, 0, 0, false, false, true),
        (0, 0, 0, 33554432, 0, false, true, false),
        (0, 64, 0, 0, 0, false, true, false),
        (0, 0, 0, 0, 47, false, true, false),
        (0, 0, 0, 0, 78, false, true, false),
        (0, 0, 0, 0, 0, true, true, false),
    ] {
        seed(
            &env,
            19750,
            SpellTooltipInput {
                mode,
                attributes_0: Some(a0),
                attributes_3: Some(a3),
                attributes_13: Some(a13),
                first_effect_kind: Some(effect),
                alternate_header: header,
                ranges: vec!["host range".into()],
                cast_ms: Some(1500),
                ..Default::default()
            },
        );
        env.exec(&format!(
            r#"
            local t=C_TooltipInfo.GetSpellByID(19750)
            local range,cast=false,false
            for _,line in ipairs(t.lines) do
                range=range or line.leftText=='host range'
                cast=cast or line.leftText==format(SPELL_CAST_TIME_SEC,1.5)
            end
            assert(range=={range} and cast=={cast}, 'R81 skip masks')
        "#
        ))
        .unwrap();
    }
}

#[test]
fn r81_spell_unknown_outputs_are_omitted_and_action_has_no_guessed_key() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q3 cooldown/charge predicates UNKNOWN; no native support for fixed binding mapping.
    env.state().borrow_mut().action_bars.insert(5, 642);
    env.exec(
        r#"
        local t=C_TooltipInfo.GetSpellByID(642)
        assert(#t.lines==2, 'unseeded spell details omitted')
        for _,line in ipairs(C_TooltipInfo.GetAction(5).lines) do
            assert(not line.leftText:find('Key bound') and line.leftText~='Not bound')
        end
        GameTooltip:SetSpellByID(642)
        assert(GameTooltip:NumLines()==2)
    "#,
    )
    .unwrap();
}
