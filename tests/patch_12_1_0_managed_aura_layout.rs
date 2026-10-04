//! 12.1.0 managed AuraContainer layout (B12b): group order, layout options,
//! layoutIndex, columns, automatic resize, and manually anchored slots.
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::{
    assert_no_lua_errors, aura, debuff, load_env, set_player_auras, settle,
};

/// `AuditLayout(container, keys)` lists every shown group button as
/// `icon@x,y` (offsets from the container's top-left, in screen order)
/// followed by `|WxH`, the container's size.
const LAYOUT_HELPERS: &str = r#"
    function AuditLayout(container, keys)
        local left, top = AuditPlain(container:GetLeft()), AuditPlain(container:GetTop())
        local entries = {}
        for _, key in ipairs(keys) do
            for _, entry in ipairs(AuditShown(container, key)) do
                entries[#entries + 1] = entry
            end
        end
        table.sort(entries, function(a, b)
            if a.y ~= b.y then return a.y > b.y end
            return a.x < b.x
        end)
        local parts = {}
        for index, entry in ipairs(entries) do
            parts[index] = string.format('%d@%d,%d', entry.icon, entry.x - left, top - entry.y)
        end
        local width, height = AuditPlain(container:GetWidth()), AuditPlain(container:GetHeight())
        return table.concat(parts, ' ') .. string.format('|%dx%d', width, height)
    end
    function AuditGroupOptions(extra)
        local options = {sortMethod = AuraContainerSortMethod.AuraInstanceIDOnly}
        for key, value in pairs(extra or {}) do options[key] = value end
        return options
    end
"#;

fn layout_env() -> wow_ui_sim::lua_api::WowLuaEnv {
    let env = load_env();
    env.exec(LAYOUT_HELPERS).unwrap();
    set_player_auras(
        &env,
        vec![
            aura(101),
            aura(102),
            aura(103),
            debuff(201, None),
            debuff(202, None),
        ],
    );
    env
}

/// (case, Lua expression creating the container, expected layout)
const LAYOUT_CASES: &[(&str, &str, &str)] = &[
    (
        "groups flow sequentially in the order they were added",
        "AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions()}, {'debuffs', 'HARMFUL', AuditGroupOptions()}})",
        "101@0,0 102@20,0 103@40,0 201@60,0 202@80,0|100x20",
    ),
    (
        "reversed add order reverses group order",
        "AuditContainer({{'debuffs', 'HARMFUL', AuditGroupOptions()}, {'buffs', 'HELPFUL', AuditGroupOptions()}})",
        "201@0,0 202@20,0 101@40,0 102@60,0 103@80,0|100x20",
    ),
    (
        "layoutIndex overrides add order",
        "AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions({layout = {layoutIndex = 2}})}, {'debuffs', 'HARMFUL', AuditGroupOptions({layout = {layoutIndex = 1}})}})",
        "201@0,0 202@20,0 101@40,0 102@60,0 103@80,0|100x20",
    ),
    (
        "element and group spacing",
        "AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions({layout = {elementSpacing = 2}})}, {'debuffs', 'HARMFUL', AuditGroupOptions({layout = {groupSpacing = 10}})}})",
        "101@0,0 102@22,0 103@44,0 201@76,0 202@96,0|116x20",
    ),
    (
        "forced new line with group line spacing",
        "AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions()}, {'debuffs', 'HARMFUL', AuditGroupOptions({layout = {forceNewLine = true, groupLineSpacing = 5}})}})",
        "101@0,0 102@20,0 103@40,0 201@0,25 202@20,25|60x45",
    ),
    (
        "element size override drives placement",
        "AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions({layout = {elementWidth = 30}})}, {'debuffs', 'HARMFUL', AuditGroupOptions()}})",
        "101@0,0 102@30,0 103@60,0 201@90,0 202@110,0|130x20",
    ),
    (
        "maximum line size wraps with line spacing",
        "AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions({layout = {lineSpacing = 4}})}}, function(c) c:SetFlowLayoutMaximumLineSize(50) end)",
        "101@0,0 102@20,0 103@0,24|40x44",
    ),
    (
        "vertical axis lays groups out in columns",
        "AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions()}, {'debuffs', 'HARMFUL', AuditGroupOptions({layout = {forceNewLine = true}})}}, function(c) c:SetFlowLayoutAxis(AnchorUtil.FlowLayoutAxis.Vertical) end)",
        "101@0,0 201@20,0 102@0,20 202@20,20 103@0,40|40x60",
    ),
];

#[test]
fn group_layout_options_position_buttons_and_size_the_container() {
    let env = layout_env();
    for (index, (_, create, _)) in LAYOUT_CASES.iter().enumerate() {
        env.exec(&format!("AuditLayout{index} = {create}")).unwrap();
    }
    settle(&env);
    let mismatches: Vec<String> = LAYOUT_CASES
        .iter()
        .enumerate()
        .filter_map(|(index, (case, _, expected))| {
            let actual = env
                .eval::<String>(&format!(
                    "return AuditLayout(AuditLayout{index}, {{'buffs', 'debuffs'}})"
                ))
                .unwrap();
            (actual != *expected).then(|| format!("{case}: expected {expected:?}, got {actual:?}"))
        })
        .collect();
    assert!(mismatches.is_empty(), "{mismatches:#?}");
    assert_no_lua_errors(&env);
}

#[test]
fn set_aura_group_layout_and_live_auras_relayout_and_resize() {
    let env = layout_env();
    env.exec(
        "AuditRelayout = AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions()}, {'debuffs', 'HARMFUL', AuditGroupOptions()}})",
    )
    .unwrap();
    settle(&env);
    let layout = || {
        env.eval::<String>("return AuditLayout(AuditRelayout, {'buffs', 'debuffs'})")
            .unwrap()
    };
    assert_eq!(layout(), "101@0,0 102@20,0 103@40,0 201@60,0 202@80,0|100x20");
    env.exec(
        "AuditAddon(function() AuditRelayout:SetAuraGroupLayout('debuffs', {forceNewLine = true, elementSpacing = 3}) end)",
    )
    .unwrap();
    settle(&env);
    assert_eq!(layout(), "101@0,0 102@20,0 103@40,0 201@0,20 202@23,20|60x40");
    // Removing every buff collapses the group and shrinks the container.
    set_player_auras(&env, vec![debuff(201, None), debuff(202, None)]);
    settle(&env);
    assert_eq!(layout(), "201@0,0 202@23,0|43x20");
    // A new aura grows it again.
    set_player_auras(&env, vec![aura(101), debuff(201, None), debuff(202, None)]);
    settle(&env);
    assert_eq!(layout(), "101@0,0 201@0,20 202@23,20|43x40");
    assert_no_lua_errors(&env);
}

#[test]
fn aura_slots_keep_addon_anchors_and_stay_out_of_group_layout() {
    let env = layout_env();
    env.exec(
        r#"
        AuditSlotLayout = AuditContainer({{'buffs', 'HELPFUL', AuditGroupOptions()}})
        AuditManualSlot = AuditAddon(function()
            local slot = AuditSlotLayout:AddAuraSlot('first-debuff', 'HARMFUL', {
                sortMethod = AuraContainerSortMethod.AuraInstanceIDOnly,
                initializeFrame = AuditInitIcon,
            })
            slot:SetPoint('TOPLEFT', AuditSlotLayout, 'BOTTOMLEFT', 7, -9)
            return slot
        end)
    "#,
    )
    .unwrap();
    settle(&env);
    settle(&env);
    let result = env
        .eval::<String>(
            r#"
            local slot = AuditManualSlot
            local point, relativeTo, relativePoint, x, y = slot:GetPoint(1)
            local left = AuditPlain(slot:GetLeft()) - AuditPlain(AuditSlotLayout:GetLeft())
            local top = AuditPlain(AuditSlotLayout:GetBottom()) - AuditPlain(slot:GetTop())
            return string.format('%s %s %s %d,%d icon=%s shown=%s at %d,%d|%s',
                AuditPlain(point), tostring(AuditPlain(relativeTo) == AuditSlotLayout),
                AuditPlain(relativePoint), AuditPlain(x), AuditPlain(y),
                tostring(AuditFrameIcon(slot)), tostring(AuditIsShown(slot)), left, top,
                AuditLayout(AuditSlotLayout, {'buffs'}))
        "#,
        )
        .unwrap();
    assert_eq!(
        result,
        "TOPLEFT true BOTTOMLEFT 7,-9 icon=201 shown=true at 7,9|101@0,0 102@20,0 103@40,0|60x20"
    );
    assert_no_lua_errors(&env);
}
