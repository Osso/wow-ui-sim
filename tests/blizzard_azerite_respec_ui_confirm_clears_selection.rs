use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};
use wow_ui_sim::lua_api::state::BagItem;

const ROOT: &str = "Blizzard_AzeriteRespecUI";
const AZERITE_ITEM_ID: u32 = 158_041;
const BAG_ID: i32 = 0;
const SLOT_INDEX: i32 = 7;

// AzeriteRespecUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_azerite_respec_ui_confirm_popup_confirms_item_and_clears_panel_slot() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                seed_respec_item(env);
                clear_recorded_lua_errors(env);

                let (popup_loaded, popup_reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_StaticPopup_Game")"#)
                    .expect("C_AddOns.LoadAddOn should return for Blizzard_StaticPopup_Game");
                assert!(
                    popup_loaded,
                    "Blizzard_StaticPopup_Game should load: {popup_reason:?}"
                );

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AzeriteRespecUI")"#)
                    .expect("C_AddOns.LoadAddOn should return for Blizzard_AzeriteRespecUI");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                let failures: String = env
                    .eval(&format!(
                        r#"
                        local failures = {{}}
                        local function expect(condition, message)
                            if not condition then
                                table.insert(failures, message)
                            end
                        end

                        local popup = StaticPopupDialogs.CONFIRM_AZERITE_EMPOWERED_RESPEC
                        expect(type(popup) == "table",
                            "CONFIRM_AZERITE_EMPOWERED_RESPEC static popup should be registered")
                        expect(type(popup.OnAccept) == "function",
                            "CONFIRM_AZERITE_EMPOWERED_RESPEC should expose OnAccept")

                        local itemLocation = {{ bagID = {BAG_ID}, slotIndex = {SLOT_INDEX} }}
                        function itemLocation:IsEqualTo(other)
                            return self == other
                        end

                        local eventFrame = CreateFrame("Frame")
                        eventFrame:RegisterEvent("AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED")
                        eventFrame:SetScript("OnEvent", function(_, event, location)
                            _G.__azerite_respec_selection_event = event
                            _G.__azerite_respec_selection_location = location
                        end)
                        local originalOnEvent = AzeriteRespecFrame.OnEvent
                        AzeriteRespecFrame.OnEvent = function(self, event, location)
                            _G.__azerite_respec_panel_event = event
                            _G.__azerite_respec_panel_location = location
                            return originalOnEvent(self, event, location)
                        end
                        local originalSetRespecItem = AzeriteRespecFrame.SetRespecItem
                        AzeriteRespecFrame.SetRespecItem = function(self, location)
                            _G.__azerite_respec_set_respec_item_called = true
                            _G.__azerite_respec_set_respec_item_arg = location
                            return originalSetRespecItem(self, location)
                        end
                        local mixinEnv = debug.getfenv(AzeriteRespecMixin.SetRespecItem)
                        mixinEnv.Item = {{
                            CreateFromItemLocation = function(_, location)
                                return {{
                                    UnlockItem = function() end,
                                    ContinueWithCancelOnItemLoad = function(_, callback)
                                        callback()
                                        return function() end
                                    end,
                                    GetItemIcon = function()
                                        return 134400
                                    end,
                                }}
                            end,
                        }}

                        AzeriteRespecFrame.respecCost = 5000
                        AzeriteRespecFrame.respecItemLocation = itemLocation
                        AzeriteRespecFrame.UpdateMoney = function() end
                        AzeriteRespecFrame.ItemSlot.Icon:Show()
                        expect(AzeriteRespecFrame:GetRespecItemLocation():IsEqualTo(itemLocation),
                            "test item location should match before confirmation")

                        popup.OnAccept(nil, {{
                            empoweredItemLocation = itemLocation,
                            respecCost = 5000,
                        }})

                        expect(_G.__azerite_respec_selection_event == "AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED",
                            "confirm should fire AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED")
                        expect(_G.__azerite_respec_selection_location == itemLocation,
                            "selection-updated event should carry the same item location")
                        expect(AzeriteRespecFrame:IsEventRegistered("AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED"),
                            "AzeriteRespecFrame should be registered for the selection-updated event")
                        expect(_G.__azerite_respec_panel_event == "AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED",
                            "selection-updated event should dispatch to AzeriteRespecFrame")
                        expect(_G.__azerite_respec_panel_location == itemLocation,
                            "AzeriteRespecFrame should receive the same item location payload")
                        expect(_G.__azerite_respec_set_respec_item_called == true,
                            "AzeriteRespecFrame should call SetRespecItem for the matching event")
                        expect(_G.__azerite_respec_set_respec_item_arg == nil,
                            "AzeriteRespecFrame should call SetRespecItem(nil) for the matching event")
                        expect(AzeriteRespecFrame.respecItemLocation == nil,
                            "AzeriteRespecFrame should clear respecItemLocation after matching event")
                        expect(not AzeriteRespecFrame.ItemSlot.Icon:IsShown(),
                            "AzeriteRespecFrame should hide the item icon after clearing the slot")

                        return table.concat(failures, "\n")
                        "#
                    ))
                    .expect("Azerite respec confirmation probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` confirm popup mismatches:\n{failures}"
                );

                let confirmed = env
                    .state()
                    .borrow()
                    .azerite_empowered
                    .last_confirmed_respec
                    .clone()
                    .expect("confirm popup should record the item location");
                assert_eq!(confirmed.bag_id, Some(BAG_ID));
                assert_eq!(confirmed.slot_index, Some(SLOT_INDEX));
                assert_eq!(confirmed.equipment_slot_index, None);

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` emitted Lua errors while checking confirm popup:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}
}

fn seed_respec_item(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.player.money = 100_000;
    state.bag_items.insert(
        (BAG_ID, SLOT_INDEX),
        BagItem {
            item_id: AZERITE_ITEM_ID,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
            owned_identity: None,
        },
    );
    state
        .azerite_empowered
        .empowered_items
        .insert(AZERITE_ITEM_ID as i32);
}
