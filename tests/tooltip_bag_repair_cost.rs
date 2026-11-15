//! VERIFIED R88n.4b publication; repair calculation remains an explicit host input.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::{WowLuaEnv, state::BagItem};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;
#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

const ORIGINAL_SOURCES: &str = r#"
    local function original(fn, suffix)
        local info = debug.getinfo(fn, 'S')
        assert(info.what == 'Lua')
        assert(info.source:gsub('\\', '/'):sub(-#suffix) == suffix)
    end
    original(GameTooltip.SetBagItem, 'Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua')
    original(TooltipDataRules.FinalizeItemTooltip, 'Blizzard_SharedXMLGame/Tooltip/TooltipDataRules.lua')
    original(GameTooltip_AddMoneyLine, 'Blizzard_GameTooltip/Mainline/GameTooltip.lua')
    assert(type(REPAIR_COST) == 'string' and REPAIR_COST ~= '')
"#;
const OBSERVE: &str = r#"
    ShowRepairCursor()
    assert(InRepairMode(), 'real repair cursor state required')
    local cases = {{slot=1, id=6948, cost=731}, {slot=2, id=7005, cost=2468}, {slot=3, id=6948}, {slot=4, id=7005}}
    for _, case in ipairs(cases) do
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetBagItem(0, case.slot)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetBagItem')
        assert(data and data.type == Enum.TooltipDataType.Item and data.id == case.id)
        assert(data.repairCost == case.cost, 'wrong top-level repairCost for slot '..case.slot)
        if case.cost then assert(data.repairCost == math.floor(data.repairCost)) end
        for _, line in ipairs(data.lines) do
            assert(line.repairCost == nil, 'repair cost must not be stored on an item line')
        end
        local found = 0
        for index = 1, GameTooltip:NumLines() do
            local line = GameTooltip:GetLeftLine(index)
            if line and line:GetText() == REPAIR_COST then
                found = found + 1
                assert(case.cost, 'zero/absent cost emitted repair output')
                local moneyLine = GameTooltip:GetLeftLine(index + 1)
                local expected = MoneyFormatterUtil.FormatMoney(case.cost, GameTooltipMoneyFormat)
                assert(moneyLine and moneyLine:GetText() == expected, 'wrong original repair money line')
            end
        end
        assert(found == (case.cost and 1 or 0), 'wrong number of repair labels')
    end
"#;

fn assert_selected_tooltip_sources() {
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for expected in ["Blizzard_SharedXMLGame", "Blizzard_GameTooltip"] {
        assert!(addons.iter().any(|(name, _)| name == expected));
    }
    let toc =
        TocFile::from_file(&ui.join("Blizzard_SharedXMLGame/Blizzard_SharedXMLGame.toc")).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    for expected in [
        "Tooltip/TooltipDataHandler.lua",
        "Tooltip/TooltipDataRules.lua",
    ] {
        assert!(toc.files.iter().enumerate().any(|(i, path)| {
            path.to_string_lossy().replace('\\', "/") == expected
                && toc.file_allows_screen(i, ScreenKind::Game)
        }));
    }
}

fn supplied_bag_item(item_id: u32) -> BagItem {
    BagItem {
        owned_identity: None,
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

fn supply_repair_inputs(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.bag_items.clear();
    sim.bag_item_repair_costs.clear();
    for (slot, item_id) in [(1, 6948), (2, 7005), (3, 6948), (4, 7005)] {
        assert!(wow_ui_sim::items::get_item(item_id).is_some());
        sim.bag_items.insert((0, slot), supplied_bag_item(item_id));
    }
    sim.bag_item_repair_costs.insert((0, 1), 731);
    sim.bag_item_repair_costs.insert((0, 2), 2468);
    sim.bag_item_repair_costs.insert((0, 3), 0);
    sim.merchant_frame_open = true;
    sim.merchant_repair_capable = true;
    sim.cursor_storage.presentation_gate_a = false;
    sim.cursor_storage.presentation_gate_b = false;
}

#[test]
fn original_bag_tooltip_repair_rule_consumes_distinct_positive_costs_and_omits_zero_absent() {
    assert_selected_tooltip_sources();
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(ORIGINAL_SOURCES)
        .expect("original selected tooltip consumer functions");
    supply_repair_inputs(&env);
    let before = {
        let sim = env.state().borrow();
        (
            sim.lua_errors.clone(),
            sim.lua_error_records.len(),
            sim.lua_error_counts.clone(),
        )
    };
    env.exec(OBSERVE)
        .expect("real bag accessor and original repair-rule money output");
    let sim = env.state().borrow();
    assert_eq!(sim.lua_errors, before.0, "original path added Lua errors");
    assert_eq!(sim.lua_error_records.len(), before.1);
    assert_eq!(sim.lua_error_counts, before.2);
}
