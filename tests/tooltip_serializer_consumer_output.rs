//! R88n.4b placement selectors observed only through original Retail tooltip
//! consumer output: the selected `TooltipDataHandler` accessor and the
//! `TooltipDataRules` rules render each field only when it sits at its native
//! TooltipData level.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]
use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::c_api::BattlePetTooltipSnapshot;
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
    original(TooltipDataRules.BattlePet, rules)
    original(BattlePetToolTip_Show, 'Blizzard_FrameXML/BattlePetTooltip.lua')
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

// INFERRED (simulator mapping): catalog `sell_price` supplies the initial price;
// this test also supplies a true result for the remaining SellPrice gates.
// VERIFIED (R88n.4b, R88p.p1): `price` sits on a type-11 line and zero adds no row.
const SELL_PRICE_ROWS: &str = r#"
    local cases = {{slot = 1, price = 1}, {slot = 2, price = 50}, {slot = 3}}
    for _, case in ipairs(cases) do
        local sellRows = 0
        for _, text in ipairs(RenderedBagTooltipRows(case.slot)) do
            if text:find(SELL_PRICE, 1, true) == 1 then
                sellRows = sellRows + 1
                assert(case.price, 'slot '..case.slot..' rendered '..text)
                local money = MoneyFormatterUtil.FormatMoney(case.price, GameTooltipMoneyFormat)
                assert(text == string.format('%s: %s', SELL_PRICE, money), text)
            end
        end
        assert(sellRows == (case.price and 1 or 0), 'slot '..case.slot..' eligible sell rows')
    end
"#;

const BATTLE_PET_ROWS: &str = r#"
    local observed = {}
    BattlePetToolTip_Show = function(...) observed[#observed + 1] = {...} end
    GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

    local function assert_pet(slot, expected, call_index)
        GameTooltip:SetBagItem(0, slot)
        local info = GameTooltip:GetPrimaryTooltipInfo()
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(info and info.getterName == 'GetBagItem', 'original bag-item accessor')
        assert(data and data.type == Enum.TooltipDataType.BattlePet, 'type-6 TooltipData')
        for field, value in pairs(expected) do
            assert(data[field] == value, field..' must be a top-level field')
        end
        assert(#observed == call_index, 'BattlePet rule call count')
        local arguments = observed[call_index]
        assert(#arguments == 7, 'BattlePet rule forwards seven arguments')
        assert(arguments[1] == expected.battlePetSpeciesID)
        assert(arguments[2] == expected.battlePetLevel)
        assert(arguments[3] == expected.battlePetBreedQuality)
        assert(arguments[4] == expected.battlePetMaxHealth)
        assert(arguments[5] == expected.battlePetPower)
        assert(arguments[6] == expected.battlePetSpeed)
        assert(arguments[7] == expected.battlePetName)
    end

    local function assert_not_pet(slot)
        local call_count = #observed
        GameTooltip:SetBagItem(0, slot)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(not data or data.type ~= Enum.TooltipDataType.BattlePet,
            'unaccepted or absent bag slot must not publish type 6')
        assert(#observed == call_count, 'unaccepted or absent slot must not call BattlePet rule')
    end

    assert_pet(1, {
        battlePetSpeciesID = 101, battlePetLevel = 7, battlePetBreedQuality = 3,
        battlePetMaxHealth = 521, battlePetPower = 47, battlePetSpeed = 39,
        battlePetName = 'Host Pet Alpha',
    }, 1)
    assert_pet(2, {
        battlePetSpeciesID = 202, battlePetLevel = 15, battlePetBreedQuality = 4,
        battlePetMaxHealth = 1037, battlePetPower = 91, battlePetSpeed = 67,
        battlePetName = 'Host Pet Beta',
    }, 2)
    assert_not_pet(3)
    assert_not_pet(4)
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

fn battle_pet_snapshots() -> [(i32, BattlePetTooltipSnapshot); 2] {
    [
        (
            1,
            BattlePetTooltipSnapshot {
                species_id: 101.0,
                level: 7.0,
                breed_quality: 3.0,
                max_health: 521.0,
                power: 47.0,
                speed: 39.0,
                name: "Host Pet Alpha".to_owned(),
            },
        ),
        (
            2,
            BattlePetTooltipSnapshot {
                species_id: 202.0,
                level: 15.0,
                breed_quality: 4.0,
                max_health: 1037.0,
                power: 91.0,
                speed: 67.0,
                name: "Host Pet Beta".to_owned(),
            },
        ),
    ]
}

// VERIFIED (R88n.4b): native type and field placement; values below are explicit host fixtures.
#[test]
fn bag_item_accessor_publishes_accepted_battle_pet_fields_to_original_consumer() {
    // Catalog rows back occupied slots only; the native acceptance is supplied separately.
    let env = consumer_env(&[(1, 6948), (2, 7005), (3, 6948)]);
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_battle_pet_tooltip_acceptance.extend([
            ((0, 1), true),
            ((0, 2), true),
            ((0, 3), false),
        ]);
        for (slot, snapshot) in battle_pet_snapshots() {
            sim.bag_battle_pet_tooltip_snapshots
                .insert((0, slot), snapshot);
        }
    }
    exec_without_new_lua_errors(&env, BATTLE_PET_ROWS);
}

#[test]
fn bag_sell_price_rule_renders_only_type_11_line_prices() {
    let items = [(1, 184318, 1), (2, 164374, 50), (3, 163924, 0)];
    for (_, item_id, price) in items {
        let item = wow_ui_sim::items::get_item(item_id).expect("catalog item");
        assert_eq!(item.sell_price, price, "backed catalog price for {item_id}");
    }
    let env = consumer_env(&items.map(|(slot, item_id, _)| (slot, item_id)));
    {
        let mut sim = env.state().borrow_mut();
        sim.item_tooltip_sell_price_gate_results
            .extend([(184318, true), (164374, true)]);
    }
    exec_without_new_lua_errors(&env, SELL_PRICE_ROWS);
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
