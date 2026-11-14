//! Retail's original container filter-menu consumer must observe the native
//! zero-return result when the bag's inventory slot is empty.

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
const ROOT: &str = "Blizzard_UIPanels_Game";

#[cfg(all(feature = "optional-tests", feature = "client-retail"))]
#[test]
fn empty_bag_profession_check_returns_no_values_in_filter_menu_consumer() {
    with_blizzard_addon_startup_shape(&[ROOT], &[], |env, loaded| {
        assert!(loaded.iter().any(|addon| addon == ROOT));

        let (return_count, can_use_filter_menu, consumer_called): (i64, bool, bool) = env
            .eval(
                r##"
                local inventorySlot = C_Container.ContainerIDToInventoryID(1)
                assert(inventorySlot ~= nil)
                assert(GetInventoryItemID("player", inventorySlot) == nil)

                local returnCount
                local calls = 0
                local original = IsInventoryItemProfessionBag
                local function observeReturns(...)
                    returnCount = select("#", ...)
                    return ...
                end
                IsInventoryItemProfessionBag = function(...)
                    calls = calls + 1
                    return observeReturns(original(...))
                end

                local canUseFilterMenu = ContainerFrame_CanContainerUseFilterMenu(1)
                return returnCount, canUseFilterMenu, calls == 1
                "##,
            )
            .expect("the selected Retail container consumer should run");

        assert_eq!(
            return_count, 0,
            "an empty inventory slot has zero API returns"
        );
        assert!(can_use_filter_menu);
        assert!(consumer_called);
    });
}
