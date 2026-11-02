use crate::common;

use common::blizzard_addon_harness::with_blizzard_addon_smoke_shape;
use common::panel_fixtures::{clear_recorded_lua_errors, recorded_lua_errors};
use wow_ui_sim::lua_api::state::{CursorInfo, CursorItemOrigin};

const ROOT: &str = "Blizzard_AzeriteRespecUI";
const VALID_ITEM_ID: u32 = 158_041;
const BAG_ID: i32 = 0;
const SLOT_INDEX: i32 = 7;
const SLOT_HANDLER_ASSERTIONS_LUA: &str = r#"
local failures = {}
local function expect(condition, message)
    if not condition then
        table.insert(failures, message)
    end
end

local location = _G.__azerite_respec_last_cursor_location
expect(type(location) == "table", "C_Cursor.GetCursorItem should return an item location table")
expect(
    location and location.bagID == _G.__azerite_respec_expected_bag_id,
    "returned cursor location should use the seeded bag id"
)
expect(
    location and location.slotIndex == _G.__azerite_respec_expected_slot_index,
    "returned cursor location should use the seeded slot index"
)
expect(
    _G.__azerite_respec_set_respec_item_self == AzeriteRespecFrame,
    "SetRespecItem should be called on AzeriteRespecFrame"
)
expect(
    _G.__azerite_respec_set_respec_item_arg == location,
    "SetRespecItem should receive the cursor item location"
)
expect(_G.__azerite_respec_clear_cursor_count == 1, "ClearCursor should run once")

return table.concat(failures, "\n")
"#;

// AzeriteRespecUI vendor handlers/templates do not ship in Wrath; modeled API probes stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_azerite_respec_ui_item_slot_drag_and_left_click_populate_from_cursor() {
    common::with_perf_lock(|| {
        common::with_timeout(240, || {
            with_blizzard_addon_smoke_shape(&[], &[], |env, _loaded| {
                clear_recorded_lua_errors(env);

                let (loaded, reason): (bool, Option<String>) = env
                    .eval(r#"return C_AddOns.LoadAddOn("Blizzard_AzeriteRespecUI")"#)
                    .expect("C_AddOns.LoadAddOn should return for Blizzard_AzeriteRespecUI");
                assert!(loaded, "`{ROOT}` should load: {reason:?}");

                install_item_slot_spies(env);

                seed_cursor_item(env);
                run_slot_handler_probe(env, "drag", "AzeriteRespecFrame.ItemSlot:OnReceiveDrag()");
                assert_cursor_cleared(env, "OnReceiveDrag");

                seed_cursor_item(env);
                run_slot_handler_probe(
                    env,
                    "left-click",
                    r#"AzeriteRespecFrame.ItemSlot:OnClick("LeftButton")"#,
                );
                assert_cursor_cleared(env, "OnClick(LeftButton)");

                let errors = recorded_lua_errors(env);
                assert!(
                    errors.is_empty(),
                    "`{ROOT}` emitted Lua errors while checking item-slot cursor handlers:\n{}",
                    errors.join("\n")
                );
            });
        });
    });
}
}

fn install_item_slot_spies(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        local slotMixinEnv = debug.getfenv(AzeriteRespecItemSlotMixin.OnReceiveDrag)
        local originalClearCursor = slotMixinEnv.ClearCursor
        local originalGetCursorItem = C_Cursor.GetCursorItem

        C_Cursor.GetCursorItem = function()
            local location = originalGetCursorItem()
            _G.__azerite_respec_last_cursor_location = location
            return location
        end

        slotMixinEnv.ClearCursor = function()
            _G.__azerite_respec_clear_cursor_count =
                (_G.__azerite_respec_clear_cursor_count or 0) + 1
            return originalClearCursor()
        end

        AzeriteRespecFrame.SetRespecItem = function(self, location)
            _G.__azerite_respec_set_respec_item_self = self
            _G.__azerite_respec_set_respec_item_arg = location
        end
        "#,
    )
    .expect("AzeriteRespec item-slot spies should install");
}

fn seed_cursor_item(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.state().borrow_mut().cursor_item = Some(CursorInfo::Item {
        item_id: VALID_ITEM_ID,
        stack_count: 1,
        origin: CursorItemOrigin::Bag {
            bag: BAG_ID,
            slot: SLOT_INDEX,
        },
        applied_transmog: None,
    });
}

fn run_slot_handler_probe(env: &wow_ui_sim::lua_api::WowLuaEnv, label: &str, call: &str) {
    reset_slot_handler_probe(env);
    env.exec(call)
        .expect("AzeriteRespec item-slot handler should run");
    let failures = slot_handler_probe_failures(env);
    assert!(
        failures.is_empty(),
        "`{ROOT}` {label} cursor handler mismatches:\n{failures}"
    );
}

fn reset_slot_handler_probe(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(&format!(
        r#"
        _G.__azerite_respec_last_cursor_location = nil
        _G.__azerite_respec_set_respec_item_self = nil
        _G.__azerite_respec_set_respec_item_arg = nil
        _G.__azerite_respec_clear_cursor_count = 0
        _G.__azerite_respec_expected_bag_id = {BAG_ID}
        _G.__azerite_respec_expected_slot_index = {SLOT_INDEX}
        "#
    ))
    .expect("AzeriteRespec item-slot probe state should reset");
}

fn slot_handler_probe_failures(env: &wow_ui_sim::lua_api::WowLuaEnv) -> String {
    env.eval(SLOT_HANDLER_ASSERTIONS_LUA)
        .expect("AzeriteRespec item-slot handler assertions should run")
}

fn assert_cursor_cleared(env: &wow_ui_sim::lua_api::WowLuaEnv, handler_name: &str) {
    let cursor_cleared = env.state().borrow().cursor_item.is_none();
    assert!(
        cursor_cleared,
        "`{ROOT}` {handler_name} should clear the Rust cursor state"
    );
}
