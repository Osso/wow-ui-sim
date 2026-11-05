use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AzeriteRespecUI";

const INSTALL_TOOLTIP_ROUTE_SPIES_LUA: &str = r#"
_G.__azerite_respec_inventory_tooltip = nil
_G.__azerite_respec_bag_tooltip = nil

GameTooltip.SetInventoryItem = function(_, unit, slot)
    _G.__azerite_respec_inventory_tooltip = { unit = unit, slot = slot }
end

GameTooltip.SetBagItem = function(_, bag, slot)
    _G.__azerite_respec_bag_tooltip = { bag = bag, slot = slot }
end
"#;

const EQUIPMENT_LOCATION_LUA: &str = r#"
{
    IsEquipmentSlot = function() return true end,
    GetEquipmentSlot = function() return 16 end,
    GetBagAndSlot = function() return nil, nil end,
}
"#;

const BAG_LOCATION_LUA: &str = r#"
{
    IsEquipmentSlot = function() return false end,
    GetEquipmentSlot = function() return nil end,
    GetBagAndSlot = function() return 2, 11 end,
}
"#;

// AzeriteRespecUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
timerunning_util_panel_fixture! {
#[test]
fn blizzard_azerite_respec_ui_item_slot_tooltip_routes_by_location_kind() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);
                load_azerite_respec_ui(env);
                install_tooltip_route_spies(env);

                assert_equipment_location_uses_inventory_tooltip(env);
                assert_bag_location_uses_bag_tooltip(env);
                assert_no_lua_errors(env);
            });
        });
    });
}
}

fn load_azerite_respec_ui(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (loaded, reason): (bool, Option<String>) = env
        .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AzeriteRespecUI")"#)
        .expect("C_AddOns.LoadAddOn should return for Blizzard_AzeriteRespecUI");
    assert!(loaded, "`{ROOT}` should load: {reason:?}");
}

fn install_tooltip_route_spies(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(INSTALL_TOOLTIP_ROUTE_SPIES_LUA)
        .expect("AzeriteRespec tooltip route spies should install");
}

fn assert_equipment_location_uses_inventory_tooltip(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (unit, slot, bag_call_count): (String, i64, i64) = env
        .eval(&format!(
            r#"
            AzeriteRespecFrame.respecItemLocation = ({EQUIPMENT_LOCATION_LUA})
            _G.__azerite_respec_inventory_tooltip = nil
            _G.__azerite_respec_bag_tooltip = nil

            AzeriteRespecFrame.ItemSlot:OnMouseEnter()

            return _G.__azerite_respec_inventory_tooltip.unit,
                _G.__azerite_respec_inventory_tooltip.slot,
                _G.__azerite_respec_bag_tooltip and 1 or 0
            "#
        ))
        .expect("equipment-location tooltip probe should run");
    assert_eq!(unit, "player");
    assert_eq!(slot, 16);
    assert_eq!(bag_call_count, 0);
}

fn assert_bag_location_uses_bag_tooltip(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let (bag, slot, inventory_call_count): (i64, i64, i64) = env
        .eval(&format!(
            r#"
            AzeriteRespecFrame.respecItemLocation = ({BAG_LOCATION_LUA})
            _G.__azerite_respec_inventory_tooltip = nil
            _G.__azerite_respec_bag_tooltip = nil

            AzeriteRespecFrame.ItemSlot:OnMouseEnter()

            return _G.__azerite_respec_bag_tooltip.bag,
                _G.__azerite_respec_bag_tooltip.slot,
                _G.__azerite_respec_inventory_tooltip and 1 or 0
            "#
        ))
        .expect("bag-location tooltip probe should run");
    assert_eq!(bag, 2);
    assert_eq!(slot, 11);
    assert_eq!(inventory_call_count, 0);
}

fn assert_no_lua_errors(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let errors = recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "`{ROOT}` emitted Lua errors while checking item-slot tooltip routing:\n{}",
        errors.join("\n")
    );
}
