//! R88n.4b placement selectors observed only through original Retail tooltip
//! consumer output: the selected `TooltipDataHandler` accessor and the
//! `TooltipDataRules` rules render each field only when it sits at its native
//! TooltipData level.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::{WowLuaEnv, state::BagItem};
use wow_ui_sim::screen::ScreenKind;
#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

const ORIGINAL_CONSUMERS: &str = r#"
    local function original(fn, suffix)
        local info = debug.getinfo(fn, 'S')
        assert(info.what == 'Lua', suffix)
        assert(info.source:gsub('\\', '/'):sub(-#suffix) == suffix, info.source)
    end
    local handler = 'Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua'
    local rules = 'Blizzard_SharedXMLGame/Tooltip/TooltipDataRules.lua'
    local tooltip = 'Blizzard_GameTooltip/Mainline/GameTooltip.lua'
    original(GameTooltip.SetBagItem, handler)
    original(TooltipDataRules.FinalizeItemTooltip, rules)
    original(TooltipDataRules.SellPrice, rules)
    original(GameTooltip_AddMoneyLine, tooltip)
    original(GameTooltip_OnTooltipAddMoney, tooltip)
"#;

// Collects rendered left-column texts after the original accessor ran.
const RENDERED_ROWS: &str = r#"
    function RenderedBagTooltipRows(slot)
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetBagItem(0, slot)
        local rows = {}
        for index = 1, GameTooltip:NumLines() do
            local line = GameTooltip:GetLeftLine(index)
            rows[index] = line and line:GetText() or ''
        end
        return rows
    end
"#;

// VERIFIED (R88n.4b): only a positive cost adds top-level `repairCost`;
// `FinalizeItemTooltip` renders it as REPAIR_COST plus one money row.
const REPAIR_COST_ROWS: &str = r#"
    ShowRepairCursor()
    assert(InRepairMode(), 'the real repair cursor must select repair mode')
    local cases = {{slot = 1, cost = 731}, {slot = 2, cost = 2468}, {slot = 3}, {slot = 4}}
    for _, case in ipairs(cases) do
        local rows, labels = RenderedBagTooltipRows(case.slot), 0
        for index, text in ipairs(rows) do
            if text == REPAIR_COST then
                labels = labels + 1
                assert(case.cost, 'slot '..case.slot..' rendered a repair label')
                local money = MoneyFormatterUtil.FormatMoney(case.cost, GameTooltipMoneyFormat)
                assert(rows[index + 1] == money, 'slot '..case.slot..' repair money row')
            end
        end
        assert(labels == (case.cost and 1 or 0), 'slot '..case.slot..' repair label count')
    end
"#;

fn assert_consumers_selected() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for expected in ["Blizzard_SharedXMLGame", "Blizzard_GameTooltip"] {
        assert!(
            addons.iter().any(|(name, _)| name == expected),
            "Retail game-screen selection must include {expected}"
        );
    }
}

fn bag_item(item_id: u32) -> BagItem {
    assert!(wow_ui_sim::items::get_item(item_id).is_some(), "{item_id}");
    BagItem {
        owned_identity: None,
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

fn consumer_env(items: &[(i32, u32)]) -> WowLuaEnv {
    assert_consumers_selected();
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(ORIGINAL_CONSUMERS)
        .expect("original selected tooltip consumers");
    env.exec(RENDERED_ROWS).expect("row observer");
    let mut sim = env.state().borrow_mut();
    sim.bag_items.clear();
    sim.bag_item_repair_costs.clear();
    for &(slot, item_id) in items {
        sim.bag_items.insert((0, slot), bag_item(item_id));
    }
    drop(sim);
    env
}

fn exec_without_new_lua_errors(env: &WowLuaEnv, lua: &str) {
    let before = env.state().borrow().lua_error_records.len();
    env.exec(lua).expect("original consumer output");
    let after = env.state().borrow().lua_error_records.len();
    assert_eq!(after, before, "the original consumer path added Lua errors");
}

#[test]
fn bag_repair_rule_renders_only_top_level_positive_repair_cost() {
    let env = consumer_env(&[(1, 6948), (2, 7005), (3, 6948), (4, 7005)]);
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_item_repair_costs.insert((0, 1), 731);
        sim.bag_item_repair_costs.insert((0, 2), 2468);
        sim.bag_item_repair_costs.insert((0, 3), 0);
        sim.merchant_frame_open = true;
        sim.merchant_repair_capable = true;
        sim.cursor_storage.presentation_gate_a = false;
        sim.cursor_storage.presentation_gate_b = false;
    }
    exec_without_new_lua_errors(&env, REPAIR_COST_ROWS);
}
