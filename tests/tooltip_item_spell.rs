#![cfg(feature = "gui")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn prepare_tooltip_layout(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_sys);
}

fn assert_color_close(actual: (f32, f32, f32), expected: (f32, f32, f32), label: &str) {
    assert!(
        (actual.0 - expected.0).abs() < 0.01
            && (actual.1 - expected.1).abs() < 0.01
            && (actual.2 - expected.2).abs() < 0.01,
        "{label} color mismatch; expected rgb=({:.3},{:.3},{:.3}), got rgb=({:.3},{:.3},{:.3})",
        expected.0,
        expected.1,
        expected.2,
        actual.0,
        actual.1,
        actual.2
    );
}

#[test]
fn test_set_item_by_id_populates_lines() {
    let env = WowLuaEnv::new().unwrap();

    // Item 229181: Ordained Forge Maul, epic (quality 4), ilvl 610, Two-Hand
    env.exec("GameTooltip:SetItemByID(229181)").unwrap();

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(num_lines > 0, "SetItemByID should populate tooltip lines");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines[0].left_text, "Ordained Forge Maul");
    // Epic quality is purple (0.64, 0.21, 0.93)
    assert!((td.lines[0].left_color.0 - 0.64).abs() < 0.01);
    assert_eq!(td.lines[1].left_text, "Item Level 610");
    assert_eq!(td.lines[2].left_text, "Two-Hand");
}

#[test]
fn test_set_item_by_id_6948_contains_hearthstone_line() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetItemByID(6948)").unwrap();

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(
        num_lines > 0,
        "SetItemByID(6948) should populate tooltip lines"
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert!(
        td.lines
            .iter()
            .any(|line| line.left_text.contains("Hearthstone")),
        "Tooltip lines for item 6948 should contain Hearthstone, got: {:?}",
        td.lines
            .iter()
            .map(|line| line.left_text.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_set_item_by_id_makes_tooltip_visible() {
    let env = WowLuaEnv::new().unwrap();

    let initially_visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(!initially_visible);

    env.exec("GameTooltip:SetItemByID(229181)").unwrap();

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(visible, "SetItemByID should make tooltip visible");
}

#[test]
fn test_set_hyperlink_populates_lines() {
    let env = WowLuaEnv::new().unwrap();

    // Standard WoW hyperlink format
    env.exec(
        r#"GameTooltip:SetHyperlink("|Hitem:229181:0:0:0:0:0:0:0:0:0|h[Ordained Forge Maul]|h")"#,
    )
    .unwrap();

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(num_lines > 0, "SetHyperlink should populate tooltip lines");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines[0].left_text, "Ordained Forge Maul");
}

#[test]
fn test_set_hyperlink_short_format() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"GameTooltip:SetHyperlink("item:229181")"#)
        .unwrap();

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(
        num_lines > 0,
        "SetHyperlink with short format should populate lines"
    );
}

#[test]
fn test_set_hyperlink_spell_link_populates_spell_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"GameTooltip:SetHyperlink(GetSpellLink(19750))"#)
        .unwrap();

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(
        num_lines >= 2,
        "SetHyperlink with a spell link should populate spell tooltip lines"
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines[0].left_text, "Flash of Light");
    assert_eq!(
        state.tooltips.get(&gt_id).and_then(|td| td.spell_id),
        Some(19750)
    );
}

#[test]
fn test_get_num_lines_returns_actual_count() {
    let env = WowLuaEnv::new().unwrap();

    let zero: i32 = env.eval("return GameTooltip:GetNumLines()").unwrap();
    assert_eq!(zero, 0, "GetNumLines should return 0 when no lines");

    env.exec("GameTooltip:SetItemByID(229181)").unwrap();

    let count: i32 = env.eval("return GameTooltip:GetNumLines()").unwrap();
    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(
        count, num_lines,
        "GetNumLines should match NumLines: got {count} vs {num_lines}"
    );
    assert!(count > 0, "GetNumLines should be > 0 after SetItemByID");
}

#[test]
fn test_set_unit_aura_populates_lines() {
    let env = WowLuaEnv::new().unwrap();
    // Player has default buffs; buff 1 should exist
    let has_buff: bool = env.eval("return UnitBuff('player', 1) ~= nil").unwrap();
    assert!(has_buff, "Player should have at least one buff");

    env.exec(r#"GameTooltip:SetUnitAura("player", 1, "HELPFUL")"#)
        .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(count > 0, "SetUnitAura should populate tooltip lines");

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(visible, "SetUnitAura should make tooltip visible");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert!(
        !td.lines[0].left_text.is_empty(),
        "First line should be the buff name"
    );
}

#[test]
fn test_set_unit_aura_colors_like_spell_tooltip() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q4: remaining-time formula UNKNOWN; final display text is host input.
    let id = env.state().borrow().player.buffs[0].aura_instance_id;
    env.state()
        .borrow_mut()
        .aura_tooltip_inputs
        .entry(id)
        .or_default()
        .remaining_text = Some("host aura time".into());
    env.exec(r#"GameTooltip:SetUnitAura("player", 1, "HELPFUL")"#)
        .unwrap();

    let highlight: (f32, f32, f32) = env
        .eval("local r,g,b = HIGHLIGHT_FONT_COLOR:GetRGB(); return r,g,b")
        .unwrap();
    let normal: (f32, f32, f32) = env
        .eval("local r,g,b = NORMAL_FONT_COLOR:GetRGB(); return r,g,b")
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let title = &td.lines[0];
    let duration = td
        .lines
        .iter()
        .find(|line| line.left_text == "host aura time")
        .expect("unit aura tooltip should include the remaining duration");
    let description = td
        .lines
        .last()
        .expect("unit aura tooltip should include a description line");

    assert_color_close(title.left_color, highlight, "unit aura title");
    assert_color_close(duration.left_color, highlight, "unit aura duration");
    assert_color_close(description.left_color, normal, "unit aura description");
    drop(state);
    env.state().borrow_mut().aura_tooltip_inputs.clear();
    env.exec(r#"GameTooltip:SetUnitAura("player", 1, "HELPFUL")"#)
        .unwrap();
    assert!(
        !env.state()
            .borrow()
            .tooltips
            .get(&gt_id)
            .unwrap()
            .lines
            .iter()
            .any(|line| line.left_text == "host aura time" || line.left_text == "1 hr")
    );
}

#[test]
fn test_set_unit_buff_populates_lines() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"GameTooltip:SetUnitBuff("player", 1)"#).unwrap();
    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(count > 0, "SetUnitBuff should populate tooltip lines");
}

#[test]
fn test_set_unit_aura_by_aura_instance_id_populates_lines() {
    let env = WowLuaEnv::new().unwrap();
    let aura_instance_id = {
        let state = env.state().borrow();
        state
            .player
            .buffs
            .first()
            .map(|aura| aura.aura_instance_id)
            .expect("Player should have at least one buff")
    };

    env.exec(&format!(
        r#"GameTooltip:SetUnitAuraByAuraInstanceID("player", {aura_instance_id})"#
    ))
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(
        count > 0,
        "SetUnitAuraByAuraInstanceID should populate tooltip lines for a valid player aura"
    );
}

#[test]
fn test_set_unit_debuff_by_aura_instance_id_does_not_show_helpful_buffs() {
    let env = WowLuaEnv::new().unwrap();
    let aura_instance_id = {
        let state = env.state().borrow();
        state
            .player
            .buffs
            .first()
            .map(|aura| aura.aura_instance_id)
            .expect("Player should have at least one buff")
    };

    env.exec(&format!(
        r#"GameTooltip:SetUnitDebuffByAuraInstanceID("player", {aura_instance_id}, "HARMFUL")"#
    ))
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(
        count, 0,
        "SetUnitDebuffByAuraInstanceID should leave the tooltip empty for helpful-only player buffs"
    );
}

#[test]
fn test_set_unit_buff_by_aura_instance_id_respects_unit() {
    let env = WowLuaEnv::new().unwrap();
    let aura_instance_id = {
        let state = env.state().borrow();
        state
            .player
            .buffs
            .first()
            .map(|aura| aura.aura_instance_id)
            .expect("Player should have at least one buff")
    };

    env.exec(&format!(
        r#"GameTooltip:SetUnitBuffByAuraInstanceID("target", {aura_instance_id})"#
    ))
    .unwrap();

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(
        count, 0,
        "SetUnitBuffByAuraInstanceID should not read player buffs when asked for another unit"
    );
}

#[test]
fn test_set_unit_aura_invalid_index_no_crash() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"GameTooltip:SetUnitAura("player", 999, "HELPFUL")"#)
        .unwrap();
    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert_eq!(count, 0, "Invalid index should leave tooltip empty");
}

#[test]
fn test_set_unit_player_populates_tooltip() {
    let env = WowLuaEnv::new().unwrap();

    let result: bool = env
        .eval(r#"return GameTooltip:SetUnit("player") == true"#)
        .unwrap();
    assert!(result, "SetUnit('player') should return true");

    let count: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(count >= 2, "Should have at least name + level lines");

    let visible: bool = env.eval("return GameTooltip:IsVisible()").unwrap();
    assert!(visible, "SetUnit should make tooltip visible");

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines[0].left_text, state.player.name);
    assert!(
        td.lines[1].left_text.contains("Level"),
        "Second line should contain level info"
    );
}

#[test]
fn test_set_unit_party_member_populates_tooltip_and_fires_event() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetPartySize(1)").unwrap();

    let result: String = env
        .eval(
            r#"
            local eventCount = 0
            local eventName, eventUnit, eventGuid
            GameTooltip:SetScript("OnTooltipSetUnit", function(self)
                eventCount = eventCount + 1
                eventName, eventUnit, eventGuid = self:GetUnit()
            end)

            local hasUnit = GameTooltip:SetUnit("party1")
            local name, unit, guid = GameTooltip:GetUnit()

            return table.concat({
                tostring(hasUnit),
                tostring(GameTooltip:NumLines()),
                tostring(name),
                tostring(unit),
                tostring(guid),
                tostring(eventCount),
                tostring(eventName),
                tostring(eventUnit),
                tostring(eventGuid),
            }, "|")
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "true|4|Thrynn|party1|Player-0000-00000002|1|Thrynn|party1|Player-0000-00000002",
        "SetUnit('party1') should populate unit lines, preserve displayed unit data, and fire OnTooltipSetUnit once"
    );
}

#[test]
fn test_set_unit_invalid_returns_false() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(r#"return GameTooltip:SetUnit("nonexistent")"#)
        .unwrap();
    assert!(!result, "SetUnit with invalid unit should return false");
}

// --- Spell tooltip tests ---

#[test]
fn test_set_spell_by_id_populates_lines() {
    let env = WowLuaEnv::new().unwrap();

    // Flash of Light (19750): has cast time (1.5s), description, and power cost
    env.exec("GameTooltip:SetSpellByID(19750)").unwrap();

    let num_lines: i32 = env.eval("return GameTooltip:NumLines()").unwrap();
    assert!(
        num_lines >= 2,
        "SetSpellByID should populate tooltip lines, got {num_lines}"
    );

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let expected_name: String = env.eval("return C_Spell.GetSpellName(19750)").unwrap();
    assert_eq!(td.lines[0].left_text, expected_name);
}

#[test]
fn test_set_spell_by_id_uses_wrapped_description_for_tooltip_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(375576)").unwrap();
    prepare_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert!(
        frame.width >= 240.0,
        "Divine Toll tooltip should not shrink to the short title width, got {}",
        frame.width
    );
}

#[test]
fn test_set_spell_by_id_colors_title_and_metadata_lines() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q2: final helper outputs, not a cost or cast-time formula.
    {
        let mut sim = env.state().borrow_mut();
        let input = sim.spell_tooltip_inputs.entry(19750).or_default();
        input.cost = Some("host MANA cost".into());
        input.cast_ms = Some(1500);
    }

    env.exec("GameTooltip:SetSpellByID(19750)").unwrap();

    let highlight: (f32, f32, f32) = env
        .eval("local r,g,b = HIGHLIGHT_FONT_COLOR:GetRGB(); return r,g,b")
        .unwrap();
    let normal: (f32, f32, f32) = env
        .eval("local r,g,b = NORMAL_FONT_COLOR:GetRGB(); return r,g,b")
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let title = &td.lines[0];
    let cost = td
        .lines
        .iter()
        .find(|line| line.left_text.contains("MANA"))
        .expect("Flash of Light tooltip should include resource cost");
    let cast = td
        .lines
        .iter()
        .find(|line| line.left_text.contains("sec cast"))
        .expect("Flash of Light tooltip should include cast time");
    let description = td
        .lines
        .last()
        .expect("Flash of Light tooltip should include a description");

    for line in [title, cost, cast] {
        assert_color_close(line.left_color, highlight, "spell metadata line");
    }
    assert_color_close(description.left_color, normal, "spell description line");
}

#[test]
fn test_set_spell_by_id_colors_cooldown_line() {
    let env = WowLuaEnv::new().unwrap();
    // R81 Q3: cooldown helper predicates UNKNOWN; supply its final text.
    env.state()
        .borrow_mut()
        .spell_tooltip_inputs
        .entry(642)
        .or_default()
        .cooldown_text = Some("host cooldown".into());

    env.exec("GameTooltip:SetSpellByID(642)").unwrap();
    let highlight: (f32, f32, f32) = env
        .eval("local r,g,b = HIGHLIGHT_FONT_COLOR:GetRGB(); return r,g,b")
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let cooldown = td
        .lines
        .iter()
        .find(|line| line.left_text.contains("cooldown"))
        .expect("Divine Shield tooltip should include cooldown line");

    assert_eq!(cooldown.left_text, "host cooldown");
    assert_color_close(cooldown.left_color, highlight, "cooldown line");
    drop(state);
    env.state().borrow_mut().spell_tooltip_inputs.clear();
    env.exec("GameTooltip:SetSpellByID(642)").unwrap();
    assert!(
        !env.state()
            .borrow()
            .tooltips
            .get(&gt_id)
            .unwrap()
            .lines
            .iter()
            .any(|line| line.left_text.contains("cooldown"))
    );
}

#[test]
fn test_get_action_omits_unsupported_binding_line() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().action_bars.insert(5, 19750);
    // R81 Q3: fixed slot-to-key mapping has no native evidence; no replacement seam.
    env.exec(
        r#"
        for _, line in ipairs(C_TooltipInfo.GetAction(5).lines) do
            assert(not line.leftText:find('Key bound') and line.leftText ~= 'Not bound')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn test_set_spell_by_id_shows_cast_time() {
    let env = WowLuaEnv::new().unwrap();

    // Flash of Light has 1500ms cast time
    env.exec("GameTooltip:SetSpellByID(19750)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    let has_cast_line = td.lines.iter().any(|l| l.left_text.contains("sec cast"));
    assert!(has_cast_line, "Should have a cast time line");
}

#[test]
fn test_set_spell_by_id_instant_cast() {
    let env = WowLuaEnv::new().unwrap();

    // Crusader Strike (35395): instant cast
    env.exec("GameTooltip:SetSpellByID(35395)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();
    assert_eq!(td.lines[0].left_text, "Crusader Strike");
    let has_instant = td.lines.iter().any(|l| l.left_text == "Instant");
    assert!(has_instant, "Instant cast spells should show 'Instant'");
}

#[test]
fn test_set_spell_by_id_replaces_damage_placeholders_in_description() {
    let env = WowLuaEnv::new().unwrap();

    env.exec("GameTooltip:SetSpellByID(31935)").unwrap();
    let avengers_shield_damage: String = env
        .eval("local function fmt(v) if math.abs(v) >= 100 or math.abs(v - math.floor(v + 0.5)) < 0.001 then return tostring(math.floor(v + 0.5)) else return string.format('%.1f', v) end end local ap = UnitAttackPower('player'); return fmt(ap * 1.55)")
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();

    assert!(
        td.lines.iter().any(|line| line
            .left_text
            .contains(&format!("{avengers_shield_damage} Holy damage"))),
        "SetSpellByID should replace spell damage placeholders with AP-scaled values, got: {:?}",
        td.lines
            .iter()
            .map(|line| line.left_text.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        td.lines.iter().all(|line| !line.left_text.contains('$')),
        "SetSpellByID should not leave raw parameter placeholders in tooltip text, got: {:?}",
        td.lines
            .iter()
            .map(|line| line.left_text.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_set_spell_by_id_replaces_shield_placeholders_from_player_health() {
    let env = WowLuaEnv::new().unwrap();

    env.state().borrow_mut().player.health_max = 120_000;
    env.exec("GameTooltip:SetSpellByID(184662)").unwrap();
    let shield_amount: i32 = env
        .eval("return math.floor(120000 * 0.30 * (1 + GetVersatilityBonus() / 100) + 0.5)")
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();

    assert!(
        td.lines.iter().any(|line| line
            .left_text
            .contains(&format!("absorbs {shield_amount} damage"))),
        "SetSpellByID should calculate shield values from player health, got: {:?}",
        td.lines
            .iter()
            .map(|line| line.left_text.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        td.lines.iter().all(|line| !line.left_text.contains('$')),
        "SetSpellByID should not leave raw parameter placeholders in shield text, got: {:?}",
        td.lines
            .iter()
            .map(|line| line.left_text.clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_set_spell_by_id_replaces_armor_placeholder_for_shield_of_the_righteous() {
    let env = WowLuaEnv::new().unwrap();
    let expected_armor: i32 = env
        .eval("local str = UnitStat('player', 1); return math.floor(str * 1.60 + 0.5)")
        .unwrap();

    env.exec("GameTooltip:SetSpellByID(53600)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let td = state.tooltips.get(&gt_id).unwrap();

    assert!(
        td.lines.iter().any(|line| line
            .left_text
            .contains(&format!("Armor by {expected_armor}"))),
        "SetSpellByID should calculate Shield of the Righteous armor from 160% primary stat, got: {:?}",
        td.lines
            .iter()
            .map(|line| line.left_text.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        td.lines
            .iter()
            .any(|line| line.left_text.contains("for 4.5")),
        "SetSpellByID should resolve Shield of the Righteous armor duration from spell 132403, got: {:?}",
        td.lines
            .iter()
            .map(|line| line.left_text.clone())
            .collect::<Vec<_>>()
    );
    let description_line = td
        .lines
        .iter()
        .find(|line| line.left_text.contains("Armor by"))
        .expect("Shield of the Righteous tooltip should include armor description");
    assert!(
        description_line.left_segments.is_empty(),
        "resolved numeric values should not invent inline color segments"
    );
}

#[path = "tooltip_item_spell/spell_lines.rs"]
mod spell_lines;

#[path = "tooltip_item_spell/inventory.rs"]
mod inventory;
