//! Retail 12.1.0 added Global API functions: state-backed behavior.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn eval_ok(env: &WowLuaEnv, code: &str) {
    let result: String = env.eval(code).unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn housing_layout_floor_range_follows_room_floors() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r#"
        if C_HousingLayout.GetHighestOccupiedFloorIndex() ~= 0 then return "empty-high" end
        if C_HousingLayout.GetLowestOccupiedFloorIndex() ~= 0 then return "empty-low" end
        return "ok"
        "#,
    );
    {
        let mut state = env.state().borrow_mut();
        state.housing.base_room_floors.insert(11, 2);
        state.housing.base_room_floors.insert(12, -1);
        state.housing.base_room_floors.insert(13, 0);
    }
    eval_ok(
        &env,
        r#"
        if C_HousingLayout.GetHighestOccupiedFloorIndex() ~= 2 then return "high" end
        if C_HousingLayout.GetLowestOccupiedFloorIndex() ~= -1 then return "low" end
        return "ok"
        "#,
    );
}

#[test]
fn housing_room_stairs_and_room_export_use_layout_rooms() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.housing.inside_owned_house = true;
        state.housing.base_room_floors.insert(11, 0);
        state.housing.base_room_floors.insert(12, 1);
        state.housing.base_room = Some(11);
        state.housing.stairwell_rooms.push(12);
    }
    eval_ok(
        &env,
        r#"
        if C_HousingLayout.RoomHasStairs(12) ~= true then return "stairs" end
        if C_HousingLayout.RoomHasStairs(11) ~= false then return "no-stairs" end
        if C_HousingLayout.RoomHasStairs("not-a-room") ~= false then return "invalid-stairs" end
        if C_HousingBlueprint.CanExportRoom(12) ~= true then return "export" end
        if C_HousingBlueprint.CanExportRoom(11) ~= false then return "base-room" end
        if C_HousingBlueprint.CanExportRoom(99) ~= false then return "unknown-room" end
        return "ok"
        "#,
    );
    env.state().borrow_mut().housing.inside_owned_house = false;
    eval_ok(
        &env,
        r#"
        if C_HousingBlueprint.CanExportRoom(12) ~= false then return "outside-house" end
        return "ok"
        "#,
    );
}

#[test]
fn housing_blueprint_export_types_follow_owned_location() {
    let env = WowLuaEnv::new().unwrap();
    let probe = r#"
        local T = Enum.HousingBlueprintType
        local function can(t) return C_HousingBlueprint.CanExportTypeFromCurrentLocation(t) end
        return string.format("%s %s %s %s %s",
            tostring(can(T.House)), tostring(can(T.Interior)), tostring(can(T.Exterior)),
            tostring(can(T.Room)), tostring(can(T.None)))
    "#;
    let outside: String = env.eval(probe).unwrap();
    assert_eq!(outside, "false false false false false");
    {
        let mut state = env.state().borrow_mut();
        state.housing.inside_owned_house = true;
        state.housing.room_player_is_in = Some(11);
    }
    let in_house: String = env.eval(probe).unwrap();
    assert_eq!(in_house, "true true false true false");
    {
        let mut state = env.state().borrow_mut();
        state.housing.inside_owned_house = false;
        state.housing.inside_owned_plot = true;
    }
    let on_plot: String = env.eval(probe).unwrap();
    assert_eq!(on_plot, "true false true false false");
}

#[test]
fn housing_blueprint_input_normalizes_hyperlinks() {
    let env = WowLuaEnv::new().unwrap();
    eval_ok(
        &env,
        r##"
        local code = C_HousingBlueprint.ExportBlueprint("abc")
        local link = C_HousingBlueprint.GetBlueprintHyperlink(code)
        if C_HousingBlueprint.UpdateBlueprintStringFromInput("  " .. code .. "\n") ~= code then
            return "trim"
        end
        if C_HousingBlueprint.UpdateBlueprintStringFromInput(link) ~= code then return "link" end
        if select("#", C_HousingBlueprint.UpdateBlueprintStringFromInput("   ")) ~= 0 then
            return "empty"
        end
        return "ok"
        "##,
    );
}

#[test]
fn housing_all_placement_budgets_map_budget_types() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.housing.max_indoor_placement_budget = Some(100);
        state.housing.max_outdoor_placement_budget = Some(50);
        state.housing.max_pet_placement_budget = Some(3);
        state.housing.spent_indoor_placement_budget = Some(20);
        state.housing.spent_outdoor_placement_budget = Some(5);
        state.housing.spent_pet_placement_budget = Some(1);
    }
    eval_ok(
        &env,
        r#"
        local maxIn, maxOut = C_HousingDecor.GetAllMaxPlacementBudgets()
        if maxIn ~= nil or maxOut ~= nil then return "outside-owned" end
        return "ok"
        "#,
    );
    env.state().borrow_mut().housing.inside_owned_house = true;
    eval_ok(
        &env,
        r#"
        local B = Enum.HousingBudgetType
        local maxIn, maxOut = C_HousingDecor.GetAllMaxPlacementBudgets()
        local spentIn, spentOut = C_HousingDecor.GetAllSpentPlacementBudgets()
        if maxIn[B.DecorPlacement] ~= 100 or maxIn[B.PetDecor] ~= 3 then return "max-in" end
        if maxOut[B.DecorPlacement] ~= 50 or maxOut[B.PetDecor] ~= nil then return "max-out" end
        if spentIn[B.DecorPlacement] ~= 20 or spentIn[B.PetDecor] ~= 1 then return "spent-in" end
        if spentOut[B.DecorPlacement] ~= 5 then return "spent-out" end
        return "ok"
        "#,
    );
}
