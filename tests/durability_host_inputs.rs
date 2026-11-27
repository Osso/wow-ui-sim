#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const DURABILITY_CASES: &str = r##"
A_Admin.EquipItem(1, 211993)
assert(not pcall(A_Admin.SetEquippedItemDurability, 1, 101, 100), "reject current above maximum")
assert(not pcall(A_Admin.SetEquippedItemDurability, 1, 0, 0), "reject zero maximum")
assert(not pcall(A_Admin.SetEquippedItemDurability, 2, 50, 100), "reject unequipped slot")

local function assert_alert(current, expected)
    A_Admin.SetEquippedItemDurability(1, current, 100)
    assert(DurabilityFrame:IsShown() and DurabilityHead:IsShown(), "head alert is visible")
    local r, g, b = DurabilityHead:GetVertexColor()
    assert(r == expected.r and g == expected.g and b == expected.b,
        "head alert color for durability " .. current)
end

assert_alert(40, INVENTORY_ALERT_COLORS[1])
assert_alert(0, INVENTORY_ALERT_COLORS[2])
"##;

#[test]
fn durability_frame_tracks_supplied_equipment_condition() {
    with_blizzard_addon_closure(&["Blizzard_DurabilityFrame"], &[], |env, loaded| {
        assert!(
            loaded
                .iter()
                .any(|addon| addon == "Blizzard_DurabilityFrame")
        );
        env.exec(DURABILITY_CASES)
            .expect("original DurabilityFrame should display supplied equipment durability");
    });
}
