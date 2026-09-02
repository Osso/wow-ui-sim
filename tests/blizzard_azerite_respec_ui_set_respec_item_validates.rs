use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};

const ROOT: &str = "Blizzard_AzeriteRespecUI";
const INVALID_ITEM_ID: i32 = 11_001;
const NO_CHOICES_ITEM_ID: i32 = 11_002;
const VALID_ITEM_ID: i32 = 11_003;

#[test]
fn blizzard_azerite_respec_ui_set_respec_item_validates_item_and_selected_powers() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                seed_azerite_respec_validation_state(env);
                clear_recorded_lua_errors(env);

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

                        local mixinEnv = debug.getfenv(AzeriteRespecMixin.SetRespecItem)
                        local errorMessages = {{}}
                        _G.__azerite_respec_locked_location = nil

                        local originalAddExternalErrorMessage = UIErrorsFrame.AddExternalErrorMessage
                        UIErrorsFrame.AddExternalErrorMessage = function(_, message)
                            table.insert(errorMessages, message)
                        end
                        mixinEnv.HelpTip = {{ Hide = function() end }}
                        mixinEnv.AzeriteEmpoweredItemDataSource = {{
                            CreateFromItemLocation = function(_, location)
                                return {{ hasSelectedPower = location.itemID == {VALID_ITEM_ID} }}
                            end,
                        }}
                        mixinEnv.AzeriteUtil = {{
                            HasSelectedAnyAzeritePower = function(azeriteItem)
                                return azeriteItem.hasSelectedPower
                            end,
                        }}
                        mixinEnv.Item = {{
                            CreateFromItemLocation = function(_, location)
                                return {{
                                    LockItem = function()
                                        _G.__azerite_respec_locked_location = location
                                    end,
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

                        local invalidLoc = {{ itemID = {INVALID_ITEM_ID}, bagID = 0, slotIndex = 5 }}
                        local noChoicesLoc = {{ itemID = {NO_CHOICES_ITEM_ID}, bagID = 0, slotIndex = 6 }}
                        local validLoc = {{ itemID = {VALID_ITEM_ID}, bagID = 0, slotIndex = 7 }}
                        AzeriteRespecFrame.respecCost = 50000

                        AzeriteRespecFrame:SetRespecItem(invalidLoc)
                        expect(errorMessages[1] == ITEM_IS_NOT_AZERITE_EMPOWERED,
                            "non-azerite item should report ITEM_IS_NOT_AZERITE_EMPOWERED")
                        expect(AzeriteRespecFrame.respecItemLocation == nil,
                            "invalid item should not set respecItemLocation")

                        AzeriteRespecFrame:SetRespecItem(noChoicesLoc)
                        expect(errorMessages[2] == AZERITE_EMPOWERED_REFORGE_NO_CHOICES_TO_UNDO,
                            "azerite item with no selected powers should report no-choices error")
                        expect(AzeriteRespecFrame.respecItemLocation == nil,
                            "no-choices item should not set respecItemLocation")

                        AzeriteRespecFrame:SetRespecItem(validLoc)
                        expect(AzeriteRespecFrame.respecItemLocation == validLoc,
                            "valid item should set respecItemLocation")
                        expect(_G.__azerite_respec_locked_location == validLoc,
                            "valid item should be locked")
                        expect(AzeriteRespecFrame.ItemSlot.Icon:IsShown(),
                            "valid item should refresh and show the item icon")

                        UIErrorsFrame.AddExternalErrorMessage = originalAddExternalErrorMessage
                        return table.concat(failures, "\n")
                        "#
                    ))
                    .expect("SetRespecItem validation probe should run");
                assert!(
                    failures.is_empty(),
                    "`{ROOT}` SetRespecItem validation mismatches:\n{failures}"
                );

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` emitted Lua errors while checking SetRespecItem validation:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}

fn seed_azerite_respec_validation_state(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.azerite_empowered.respec_cost = 50_000;
    state.player.money = 100_000;
    state
        .azerite_empowered
        .empowered_items
        .insert(NO_CHOICES_ITEM_ID);
    state
        .azerite_empowered
        .empowered_items
        .insert(VALID_ITEM_ID);
}
